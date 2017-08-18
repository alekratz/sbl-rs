use ir::*;
use vm::*;
use errors::*;
use compile::{Compile, Optimize};
use std::collections::HashMap;

pub struct CompileBytes {
    fun_table: IRFunTable,
}

impl CompileBytes {
    pub fn new(fun_table: IRFunTable) -> Self {
        CompileBytes { fun_table }
    }
}

impl Compile for CompileBytes {
    type Out = FunTable;
    fn compile(self) -> Result<Self::Out> {
        Ok(self.fun_table
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<FunTable>())
    }
}

/// An optimizer that inlines functions.
pub struct OptimizeBcInline {
    fun_table: FunTable,
    to_inline: HashMap<String, BcBody>,
}

impl Optimize for OptimizeBcInline {
    type Out = FunTable;

    fn optimize(mut self) -> Self::Out {
        self.determine_inlines();
        self.replace_inlines();

        self.fun_table
    }
}

impl OptimizeBcInline {
    pub fn new(fun_table: FunTable) -> Self {
        OptimizeBcInline {
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
                let ref fun_body = fun.as_user_fun().body;
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
                    fun.as_user_fun().body.iter().any(
                        |bc| self.is_inline_call(bc),
                    )
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
                let ref body = (fun as &Fun).as_user_fun().body;
                for bc in body {
                    if self.is_inline_call(bc) {
                        let call_name = bc.clone().val.unwrap().as_ident().to_string();
                        new_body.append(&mut self.to_inline.get(&call_name).unwrap().clone());
                    } else {
                        new_body.push(bc.clone());
                    }
                }
            }

            let tokens = self.fun_table
                .get(&fname)
                .unwrap()
                .as_user_fun()
                .tokens
                .clone();

            // replace the function with the new body
            self.fun_table.insert(
                fname.clone(),
                Fun::UserFun(UserFun::new(fname, new_body, tokens)),
            );
        }
    }
}
