use vm::*;
use syntax::*;
use errors::*;
use compile::{Compile, Optimize};
use std::rc::Rc;
use std::collections::HashMap;

/// A BoringTable is the predecessor to a FunTable; function names are first
/// gathered, and then filled in.
type BoringTable = HashMap<String, Option<Fun>>;

pub struct CompileBytes {
    ast: AST,
    fun_table: BoringTable,
}

impl Compile for CompileBytes {
    type Out = FunTable;
    /// Consumes the compiler, producing a `FunTable` on success or message on
    /// error.
    fn compile(mut self) -> Result<Self::Out> {
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
                .map(|(k, v)| (k, v.unwrap()))
                .collect(),
        )
    }
}

impl CompileBytes {
    pub fn new(ast: AST) -> Self {
        CompileBytes {
            ast,
            fun_table: BoringTable::new(),
        }
    }

    /// Appends a set of builtin functions to the funtable. Overwrites any
    /// functions that have been defined already.
    pub fn builtins(mut self, builtins: &'static HashMap<&'static str, BuiltinFun>) -> Self {
        for (k, v) in builtins.into_iter().map(|(k, v)| {
            (k.to_string(), Some(Fun::BuiltinFun(v)))
        })
        {
            self.fun_table.insert(k, v);
        }
        self
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

    fn compile_block(&self, block: &Block, jmp_offset: usize) -> Result<BcBody> {
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
                        Some(Bc::jmpz(br.tokens().into(), Val::Int((jmp_offset + end_addr) as i64)));
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
                        Some(Bc::jmpz(lp.tokens().into(), Val::Int((jmp_offset + end_addr) as i64)));
                }
            }
        }
        Ok(body.into_iter().map(Option::unwrap).collect())
    }

    fn compile_stack_stmt(&self, stmt: &StackStmt) -> Result<BcBody> {
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

    fn compile_item_push(&self, item: &Item) -> Result<BcBody> {
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

    fn compile_local_stack(&self, item: &Item) -> Result<BcBody> {
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

/// An optimizer that inlines functions.
pub struct OptimizeInline {
    fun_table: FunTable,
    to_inline: HashMap<String, BcBody>,
}

impl Optimize for OptimizeInline {
    type Out = FunTable;

    fn optimize(mut self) -> Self::Out {
        self.determine_inlines();
        self.replace_inlines();

        self.fun_table
    }
}

impl OptimizeInline {
    pub fn new(fun_table: FunTable) -> Self {
        OptimizeInline {
            fun_table,
            to_inline: HashMap::new(),
        }
    }
    /// Determines whether a given function should be inlined.
    fn should_inline(fun: &Fun) -> bool {
        const SKIP: &[&'static str] = &["main"]; // function names to skip and not inline
        if let &Fun::UserFun(ref fun) = fun as &Fun {
            !SKIP.contains(&fun.name.as_str()) &&
                !fun.body.iter().any(|bc| bc.bc_type == BcType::Call)
        } else {
            false
        }
    }

    fn is_inline_call(&self, bc: &Bc) -> bool {
        if let &Some(Val::Ident(ref fname)) = &bc.val {
            bc.bc_type == BcType::Call && self.to_inline.contains_key(fname)
        } else {
            false
        }
    }

    /// Determines which functions to inline.
    /// Functions are inlined if they don't call another function.
    fn determine_inlines(&mut self) {
        for (ref fname, ref fun) in &self.fun_table {
            if Self::should_inline(fun) {
                let ref fun_body = fun.user_fun().body;
                // this gets all except the last instruction, which is the 'RET' instruction which
                // messes things up a little bit.
                let body_clone = fun_body
                    .clone()
                    .iter()
                    .cloned()
                    .take(fun_body.len() - 1)
                    .collect::<Vec<_>>();
                self.to_inline.insert(fname.to_string(), body_clone);
            }
        }
    }

    fn replace_inlines(&mut self) {
        let mut to_optimize = vec![];
        // this section determines which functions we're going to apply optimizations to
        {
            for (ref fname, ref fun) in &self.fun_table {
                // if this fname is *not* in the list of things to inline
                if !self.to_inline.contains_key(fname.as_str())
                    // this checks if a user function has a call to one of the inlines
                    && fun.is_user_fun() &&
                    fun.user_fun().body.iter().any(|bc| self.is_inline_call(bc))
                {
                    to_optimize.push(fname.to_string());
                }
            }
        }

        // this section applies optimizations
        for fname in to_optimize {
            let mut new_body = vec![];
            {
                let fun = self.fun_table.get(&fname).unwrap();
                let ref body = (fun as &Fun).user_fun().body;
                for bc in body {
                    if self.is_inline_call(bc) {
                        let call_name = bc.clone().val.unwrap().ident().to_string();
                        new_body.append(&mut self.to_inline.get(&call_name).unwrap().clone());
                    } else {
                        new_body.push(bc.clone());
                    }
                }
            }

            let tokens = self.fun_table
                .get(&fname)
                .unwrap()
                .user_fun()
                .tokens
                .clone();

            // replace the function with the new body
            self.fun_table.insert(
                fname.clone(),
                Fun::UserFun(Rc::new(UserFun::new(fname, new_body, tokens))),
            );
        }

    }
}
