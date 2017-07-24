use vm::*;
use syntax::*;
use errors::*;
use std::rc::Rc;

/// A BoringTable is the predecessor to a FunTable; function names are first
/// gathered, and then filled in.
type BoringTable = HashMap<String, Option<Fun>>;

pub struct Compiler<'ast> {
    ast: &'ast AST,
    fun_table: BoringTable,
}

impl<'ast> Compiler<'ast> {
    pub fn new(ast: &'ast AST) -> Self {
        Compiler {
            ast,
            fun_table: BUILTINS
                .iter()
                .map(|(k, v)| (k.to_string(), Some(Fun::BuiltinFun(v))))
                .collect::<HashMap<String, _>>(),
        }
    }

    /// Consumes the compiler, producing a `FunTable` on success or message on
    /// error.
    pub fn compile(mut self) -> Result<FunTable> {
        self.fill_boring_table()?;
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
                let mut block = self.compile_block(&fun.block, 0)?;
                block.push(Bc::ret(fun.tokens().into()));
                let built_fun = UserFun::new(fun_name, block, fun.tokens().into());

                self.fun_table.insert(
                    built_fun.name.clone(),
                    Some(Fun::UserFun(Rc::new(built_fun))),
                );
            }
        }
        Ok(
            self.fun_table
                .into_iter()
                .map(|(k, v)| (k, Rc::new(v.unwrap())))
                .collect(),
        )
    }

    fn fill_boring_table(&mut self) -> Result<()> {
        // TODO : refactor and de-duplicate
        for top in &self.ast.ast {
            match top {
                &TopLevel::FunDef(ref fun) => {
                    if let Some(other) = self.fun_table.get(&fun.name) {
                        match *other {
                            Some(Fun::ForeignFun(_)) |
                            None => {
                                // None means it's a function we inserted earlier
                                return (Err(
                                    format!(
                                        "function `{}` has already been defined",
                                        &fun.name
                                    ).into(),
                                ) as Result<_>)
                                    .chain_err(|| fun.range());
                            }
                            _ => {}
                        }
                    }
                    self.fun_table.insert(fun.name.clone(), None);
                }
                &TopLevel::Foreign(ref foreign) => {
                    for frn_fun in &foreign.functions {
                        if let Some(other) = self.fun_table.get(&frn_fun.name) {
                            match *other {
                                Some(Fun::ForeignFun(_)) |
                                None => {
                                    // None means it's a function we inserted earlier
                                    return (Err(
                                        format!(
                                            "function `{}` has already been defined",
                                            &frn_fun.name
                                        ).into(),
                                    ) as Result<_>)
                                        .chain_err(|| frn_fun.range());
                                }
                                _ => {}
                            }
                        }
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

    fn compile_block(&self, block: &'ast Block, jmp_offset: usize) -> Result<BcBody> {
        let mut body = vec![];
        for stmt in &block.block {
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
                    body.append(&mut self.compile_block(
                        &br.block,
                        jmp_offset + start_addr + 1,
                    )?
                        .into_iter()
                        .map(Some)
                        .collect());
                    let end_addr = if let &Some(ref el) = &br.el_stmt {
                        let end_addr = body.len();
                        body.push(None);
                        body.append(&mut self.compile_block(
                            &el.block,
                            jmp_offset + end_addr + 1,
                        )?
                            .into_iter()
                            .map(Some)
                            .collect());
                        body[end_addr] =
                            Some(Bc::jmp(br.tokens().into(), Val::Int(body.len() as i64)));
                        end_addr + 1
                    } else {
                        body.len()
                    };
                    body[start_addr] =
                        Some(Bc::jmpz(br.tokens().into(), Val::Int(end_addr as i64)));
                }
                Stmt::Loop(ref lp) => {
                    let start_addr = body.len();
                    body.push(None);
                    body.append(&mut self.compile_block(
                        &lp.block,
                        jmp_offset + start_addr + 1,
                    )?
                        .into_iter()
                        .map(Some)
                        .collect());
                    body.push(Some(
                        Bc::jmp(lp.tokens().into(), Val::Int(start_addr as i64)),
                    ));
                    let end_addr = body.len();
                    body[start_addr] =
                        Some(Bc::jmpz(lp.tokens().into(), Val::Int(end_addr as i64)));
                }
            }
        }
        Ok(body.into_iter().map(Option::unwrap).collect())
    }

    fn compile_stack_stmt(&self, stmt: &'ast StackStmt) -> Result<BcBody> {
        let mut body = BcBody::new();
        for action in &stmt.stack_actions {
            match *action {
                StackAction::Push(ref i) => body.append(&mut self.compile_item_push(i)?),
                StackAction::Pop(_, ref i) => {
                    if matches!(i.item_type, ItemType::Int(_)) {
                        body.push(Bc::popn(action.tokens().into(), i.into()))
                    } else {
                        body.push(Bc::pop(action.tokens().into(), i.into()))
                    }
                }
            }
        }
        Ok(body)
    }

    fn compile_item_push(&self, item: &'ast Item) -> Result<BcBody> {
        match item.item_type {
            ItemType::Stack(_) => self.compile_local_stack(item),
            ItemType::Ident(ref ident) => {
                if self.fun_table.contains_key(ident) || BUILTINS.contains_key(ident.as_str()) {
                    Ok(vec![Bc::call(item.tokens().into(), item.into())])
                } else {
                    Ok(vec![Bc::load(item.tokens().into(), item.into())])
                }
            }
            _ => Ok(vec![Bc::push(item.tokens().into(), item.into())]),
        }
    }

    fn compile_local_stack(&self, item: &'ast Item) -> Result<BcBody> {
        assert_matches!(item.item_type, ItemType::Stack(_));
        let items = if let ItemType::Stack(ref stack) = item.item_type {
            stack
        } else {
            unreachable!()
        };
        // const stacks can just be pushed themselves
        if items.iter().all(Item::is_const) {
            Ok(vec![Bc::push(item.tokens().into(), item.into())])
        } else {
            let mut body = vec![Bc::push(item.tokens().into(), Val::Stack(vec![]))];
            for item in items {
                body.append(&mut self.compile_item_push(item)?);
                body.push(Bc::pushl(item.tokens().into()));
            }
            Ok(body)
        }
    }
}
