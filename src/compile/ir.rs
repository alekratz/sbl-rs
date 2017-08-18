use ir::*;
use vm::*;
use syntax::*;
use errors::*;
use compile::{Compile, BakeIR};
use std::collections::HashMap;

/*
 * IR compiler
 */

/// A BoringTable is the predecessor to a FunTable; function names are first
/// gathered, and then filled in.
type BoringTable = HashMap<String, Option<IRFun>>;

pub struct CompileIR<'ast> {
    ast: &'ast AST,
    fun_table: BoringTable,
}

impl<'ast> Compile for CompileIR<'ast> {
    type Out = IRFunTable;
    /// Consumes the compiler, producing a `FunTable` on success or message on
    /// error.
    fn compile(mut self) -> Result<Self::Out> {
        // set up the function table
        self.fill_boring_table()?;

        // fill the entries for the function table
        for top in &self.ast.ast {
            if let &TopLevel::FunDef(ref fun) = top {
                let fun_name = fun.name.clone();
                {
                    let fun_entry = self.fun_table.get(&fun_name).expect(
                        "got function with name that was not filled out",
                    );
                    assert!(
                        fun_entry.is_none(),
                        "found duplicate function that was not caught in fill_boring_table(): `{}`",
                        fun_name
                    );
                }
                let mut block = {
                    let block_compiler = CompileIRBlock::new(&self.fun_table, &fun.block, 0);
                    block_compiler.compile()?
                };
                block.push(IR::ret(fun.tokens().into()));
                let built_fun = IRUserFun::new(fun_name, block, fun.tokens().into());

                self.fun_table.insert(
                    built_fun.name.clone(),
                    Some(IRFun::UserFun(built_fun)),
                );
            }
        }

        // run bake blocks
        let fun_table = self.fun_table
            .into_iter()
            .map(|(k, v)| (k, v.unwrap()))
            .collect();

        let bake = BakeIR::new(fun_table);
        bake.compile()
    }
}

impl<'ast> CompileIR<'ast> {
    pub fn new(ast: &'ast AST) -> Self {
        CompileIR {
            ast,
            fun_table: BoringTable::new(),
        }
    }

    /// Appends a set of builtin functions to the funtable. Overwrites any
    /// functions that have been defined already.
    pub fn builtins(mut self, builtins: &'static HashMap<&'static str, BuiltinFun>) -> Self {
        for (k, v) in builtins.into_iter().map(|(k, v)| {
            (k.to_string(), Some(IRFun::BuiltinFun(v)))
        })
        {
            self.fun_table.insert(k, v);
        }
        self
    }

    /// Fills the function table with null values of functions that have yet to be compiled.
    fn fill_boring_table(&mut self) -> Result<()> {
        /// Utility function that checks if a function has already been defined in the
        /// table.
        fn check_defined(name: &str, fun_table: &BoringTable) -> Result<()> {
            if let Some(other) = fun_table.get(name) {
                match *other {
                    Some(IRFun::ForeignFun(_)) |
                    None => {
                        // None means it's a function we inserted earlier
                        return Err(
                            format!("function `{}` has already been defined", name).into(),
                        ) as Result<_>;
                    }
                    _ => {}
                }
            }
            Ok(())
        }

        for top in &self.ast.ast {
            match top {
                &TopLevel::FunDef(ref fun) => {
                    check_defined(&fun.name, &self.fun_table).chain_err(
                        || fun.range(),
                    )?;
                    self.fun_table.insert(fun.name.clone(), None);
                }
                &TopLevel::Foreign(ref foreign) => {
                    for frn_fun in &foreign.functions {
                        check_defined(&frn_fun.name, &self.fun_table).chain_err(
                            || {
                                frn_fun.range()
                            },
                        )?;
                        self.fun_table.insert(
                            frn_fun.name.clone(),
                            Some(IRFun::ForeignFun(frn_fun.clone())),
                        );
                    }
                }

                _ => panic!("got unprocessed top-level: {:#?}", top),
            }
        }
        Ok(())
    }
}

pub struct CompileIRBlock<'ft, 'b> {
    pub fun_table: &'ft BoringTable,
    pub block: &'b Block,
    pub jmp_offset: usize,
}

impl<'ft, 'b> CompileIRBlock<'ft, 'b> {
    pub fn new(fun_table: &'ft BoringTable, block: &'b Block, jmp_offset: usize) -> Self {
        CompileIRBlock {
            fun_table,
            block,
            jmp_offset,
        }
    }

    fn compile_stack_stmt(&self, stmt: &StackStmt) -> Result<IRBody> {
        let mut body = IRBody::new();
        for action in &stmt.stack_actions {
            match *action {
                StackAction::Push(ref i) => body.append(&mut self.compile_item_push(i)?),
                StackAction::Pop(_, ref i) => {
                    if matches!(i.item_type, ItemType::Int(_)) {
                        body.push(IR::popn(action.tokens().into(), i.into()))
                    } else {
                        body.push(IR::pop(action.tokens().into(), i.into()))
                    }
                }
            }
        }
        Ok(body)
    }

    fn compile_item_push(&self, item: &Item) -> Result<IRBody> {
        match item.item_type {
            ItemType::Stack(_) => self.compile_local_stack(item),
            ItemType::Ident(ref ident) => {
                if self.fun_table.contains_key(ident) || BUILTINS.contains_key(ident.as_str()) {
                    Ok(vec![IR::call(item.tokens().into(), item.into())])
                } else {
                    Ok(vec![IR::load(item.tokens().into(), item.into())])
                }
            }
            _ => Ok(vec![IR::push(item.tokens().into(), item.into())]),
        }
    }

    fn compile_local_stack(&self, item: &Item) -> Result<IRBody> {
        assert_matches!(item.item_type, ItemType::Stack(_));
        let items = if let ItemType::Stack(ref stack) = item.item_type {
            stack
        } else {
            unreachable!()
        };
        // const stacks can just be pushed themselves
        if items.iter().all(Item::is_const) {
            Ok(vec![IR::push(item.tokens().into(), item.into())])
        } else {
            let mut body = vec![IR::push(item.tokens().into(), IRVal::Stack(vec![]))];
            for item in items {
                body.append(&mut self.compile_item_push(item)?);
                body.push(IR::pushl(item.tokens().into()));
            }
            Ok(body)
        }
    }
}

impl<'ft, 'b> Compile for CompileIRBlock<'ft, 'b> {
    type Out = IRBody;
    fn compile(self) -> Result<Self::Out> {
        let mut body = vec![];
        let jmp_offset = self.jmp_offset;
        for stmt in &self.block.block {
            match *stmt {
                Stmt::Stack(ref s) => {
                    body.append(&mut self.compile_stack_stmt(s)?
                        .into_iter()
                        .map(Some)
                        .collect())
                }
                Stmt::Br(ref br) => {
                    let start_addr = body.len();
                    body.push(None); // placeholder for later
                    let block_compiler =
                        CompileIRBlock::new(self.fun_table, &br.block, jmp_offset + start_addr + 1);
                    body.append(&mut block_compiler
                        .compile()?
                        .into_iter()
                        .map(Some)
                        .collect());
                    let end_addr = if let &Some(ref el) = &br.el_stmt {
                        let end_addr = body.len();
                        body.push(None);
                        let block_compiler = CompileIRBlock::new(
                            self.fun_table,
                            &el.block,
                            jmp_offset + start_addr + 1,
                        );
                        body.append(&mut block_compiler
                            .compile()?
                            .into_iter()
                            .map(Some)
                            .collect());
                        body[end_addr] =
                            Some(IR::jmp(br.tokens().into(), IRVal::Int(body.len() as i64)));
                        end_addr + 1
                    } else {
                        body.len()
                    };
                    body[start_addr] = Some(IR::jmpz(
                        br.tokens().into(),
                        IRVal::Int((jmp_offset + end_addr) as i64),
                    ));
                }
                Stmt::Loop(ref lp) => {
                    let start_addr = body.len();
                    body.push(None);
                    let block_compiler =
                        CompileIRBlock::new(self.fun_table, &lp.block, jmp_offset + start_addr + 1);
                    body.append(&mut block_compiler
                        .compile()?
                        .into_iter()
                        .map(Some)
                        .collect());
                    body.push(Some(
                        IR::jmp(lp.tokens().into(), IRVal::Int(start_addr as i64)),
                    ));
                    let end_addr = body.len();
                    body[start_addr] = Some(IR::jmpz(
                        lp.tokens().into(),
                        IRVal::Int((jmp_offset + end_addr) as i64),
                    ));
                }
                Stmt::Bake(ref block) => {
                    body.push(Some(IR::bake(
                        block.tokens().into(),
                        IRVal::BakeBlock(block.block.clone()),
                    )))
                }

            }
        }
        Ok(body.into_iter().map(Option::unwrap).collect())
    }
}
