use prelude::*;
use std::collections::BTreeMap;

/*
 * IR compiler
 */

/// A BoringTable is the predecessor to a BCFunTable; function names are first
/// gathered, and then filled in.
type BoringTable = BTreeMap<String, Option<IRFun>>;

pub struct CompileIR<'ast> {
    pub ast: &'ast AST,
    pub fun_table: BoringTable,
}

impl<'ast> Compile for CompileIR<'ast> {
    type Out = IRFunTable;
    /// Consumes the compiler, producing a `BCFunTable` on success or message on
    /// error.
    fn compile(mut self) -> Result<Self::Out> {
        // set up the function table
        self.fill_boring_table()?;

        // fill the entries for the function table
        for top in &self.ast.ast {
            if let &TopLevel::BCFunDef(ref fun) = top {
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
                    let mut label_offset = 0;
                    let block_compiler = CompileIRBlock::new(&self.fun_table, &fun.block, &mut label_offset);
                    block_compiler.compile()?
                };
                block.push(IR::ret(fun.tokens().into()));
                let built_fun = IRUserFun::new(fun_name, block, fun.tokens().into());

                self.fun_table.insert(
                    built_fun.name.clone(),
                    Some(Fun::UserFun(built_fun)),
                );
            }
        }

        Ok(self.fun_table
            .into_iter()
            .map(|(k, v)| (k, v.unwrap()))
            .collect())
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
    pub fn builtins(mut self, builtins: &'static BTreeMap<&'static str, BuiltinFun>) -> Self {
        for (k, v) in builtins.into_iter().map(|(k, v)| {
            (k.to_string(), Some(Fun::BuiltinFun(v)))
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
                    Some(Fun::ForeignFun(_)) |
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
                &TopLevel::BCFunDef(ref fun) => {
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
                            Some(Fun::ForeignFun(frn_fun.clone())),
                        );
                    }
                }

                _ => panic!("got unprocessed top-level: {:#?}", top),
            }
        }
        Ok(())
    }
}

pub struct CompileIRBlock<'ft, 'b, 'l> {
    pub fun_table: &'ft BoringTable,
    pub block: &'b Block,
    pub label_offset: &'l mut usize,
}

impl<'ft, 'b, 'l> CompileIRBlock<'ft, 'b, 'l> {
    pub fn new(fun_table: &'ft BoringTable, block: &'b Block, label_offset: &'l mut usize) -> Self {
        CompileIRBlock {
            fun_table,
            block,
            label_offset,
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

impl<'ft, 'b, 'l> Compile for CompileIRBlock<'ft, 'b, 'l> {
    type Out = IRBody;
    fn compile(self) -> Result<Self::Out> {
        let mut body = vec![];
        for stmt in &self.block.block {
            match *stmt {
                Stmt::Stack(ref s) => {
                    body.append(&mut self.compile_stack_stmt(s)?
                        .into_iter()
                        .map(Some)
                        .collect())
                }
                Stmt::Br(ref br) => {
                    //
                    // Branches are structured as such:
                    //      jmpz a
                    //      ; br body
                    // a:
                    //      ...
                    //
                    // Branches with 'el' statements are structured as such:
                    //
                    //      jmpz a
                    //      ; br body
                    //      jmp b
                    // a:
                    //      ; el body
                    // b:
                    //
                    // Compile the 'br' block
                    //
                    {
                        let jmp_addr = body.len();
                        // Push the conditional jump
                        body.push(None);
                        {
                            let block_compiler = CompileIRBlock::new(self.fun_table, &br.block, self.label_offset);
                            body.append(&mut block_compiler
                                        .compile()?
                                        .into_iter()
                                        .map(Some)
                                        .collect());
                        }
                        // Fill in the jump in the slot we made earlier, and create the label
                        *self.label_offset += 1;
                        let jmp_label = IRVal::Int(*self.label_offset as i64);
                        body[jmp_addr] = Some(IR::jmpz(
                                br.tokens().into(), jmp_label.clone()));
                        // If there's an else statement, it will make its own jump label so that
                        // the instruction order is correct (and we're not jumping before another
                        // jump)
                        if br.el_stmt.is_none() {
                            body.push(Some(IR::label(br.tokens().into(), jmp_label)));
                        }
                    }

                    //
                    // Compile the 'el' block, if necessary
                    //
                    if let &Some(ref el) = &br.el_stmt {
                        // Push the unconditional jump statement, followed by the label.
                        //
                        let jmp_addr = body.len();
                        body.push(None);
                        // don't increment the label here, because we're using the previous label
                        // from above.
                        let jmp_label = IRVal::Int(*self.label_offset as i64);
                        body.push(Some(IR::label(br.tokens().into(), jmp_label)));

                        // Compile the 'el' block
                        {
                            let block_compiler = CompileIRBlock::new(
                                self.fun_table,
                                &el.block,
                                self.label_offset
                            );
                            body.append(&mut block_compiler
                                .compile()?
                                .into_iter()
                                .map(Some)
                                .collect());
                        }

                        // Create the label above and fill in the previous jump
                        *self.label_offset += 1;
                        let jmp_label = IRVal::Int(*self.label_offset as i64);
                        body[jmp_addr] = Some(IR::jmp(el.tokens().into(), jmp_label.clone()));
                        body.push(Some(IR::label(el.tokens().into(), jmp_label)));
                    }
                }
                Stmt::Loop(ref lp) => {
                    //
                    // Loops are structured as such:
                    // a:
                    //      jmpz b
                    //      ; loop body
                    //      jmp a
                    // b:
                    //
                    // Create the initial label and jump
                    *self.label_offset += 1;
                    let jmp_label = IRVal::Int(*self.label_offset as i64);
                    body.push(Some(IR::label(lp.tokens().into(), jmp_label.clone())));
                    let jmp_addr = body.len();
                    body.push(None);
                    //
                    // Compile the 'loop' block
                    //
                    {
                        let block_compiler =
                            CompileIRBlock::new(self.fun_table, &lp.block, self.label_offset);
                        body.append(&mut block_compiler
                            .compile()?
                            .into_iter()
                            .map(Some)
                            .collect());
                    }
                    // Create the jump to the next check
                    body.push(Some(
                        IR::jmp(lp.tokens().into(), jmp_label),
                    ));
                    // Create the label and fill in the previous jump
                    *self.label_offset += 1;
                    let jmp_label = IRVal::Int(*self.label_offset as i64);
                    body[jmp_addr] = Some(IR::jmpz(
                        lp.tokens().into(),
                        jmp_label.clone()
                    ));
                    body.push(Some(IR::label(lp.tokens().into(), jmp_label.clone())));
                }
                Stmt::Bake(ref block) => {
                    //
                    // Bake blocks are special. They get executed at the time that bytecode is
                    // compiled. Thus, they are simply opaque instructions that get resolved later.
                    //
                    body.push(Some(IR::bake(
                        block.tokens().into(),
                        IRVal::BakeBlock({
                            let bake_compiler = CompileIRBlock::new(self.fun_table, &block.block, self.label_offset);
                            bake_compiler.compile()?
                        }),
                    )))
                }
            }
        }
        Ok(body.into_iter().map(Option::unwrap).collect())
    }
}
