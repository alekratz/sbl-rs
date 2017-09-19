use prelude::*;
use std::collections::BTreeMap;

/// An optimizer that inlines functions.
pub struct Inline {
    fun_table: BCFunTable,
    to_inline: BTreeMap<String, BCBody>,
}

impl Optimize for Inline {
    type Out = BCFunTable;

    fn optimize(mut self) -> Self::Out {
        self.determine_inlines();
        self.replace_inlines();

        self.fun_table
    }
}

impl Inline {
    pub fn new(fun_table: BCFunTable) -> Self {
        Inline {
            fun_table,
            to_inline: BTreeMap::new(),
        }
    }
    /// Determines whether a given function should be inlined.
    fn should_inline(fun: &BCFun) -> bool {
        const SKIP: &[&'static str] = &["main"]; // function names to skip and not inline
        if let &Fun::UserFun(ref fun) = fun as &BCFun {
            !SKIP.contains(&fun.name.as_str()) &&
                !fun.body.iter().any(|bc| bc.bc_type == BCType::Call)
        } else {
            false
        }
    }

    fn is_inline_call(&self, bc: &BC) -> bool {
        if let &Some(BCVal::Ident(ref fname)) = &bc.val {
            bc.bc_type == BCType::Call && self.to_inline.contains_key(fname)
        } else {
            false
        }
    }

    /// Determines which functions to inline.
    /// BCFunctions are inlined if they don't call another function.
    fn determine_inlines(&mut self) {
        for (ref fname, ref fun) in &self.fun_table {
            if Self::should_inline(fun) {
                let ref fun_body = fun.as_user_fun().body;
                // this gets all except the last instruction, which is the 'RET' instruction which
                // messes things up a little bit.
                let body_clone = fun_body
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
                let ref body = (fun as &BCFun).as_user_fun().body;
                for bc in body {
                    if self.is_inline_call(bc) {
                        let call_name = bc.clone().val.unwrap().as_ident().to_string();
                        new_body.append(&mut self.to_inline.get(&call_name).unwrap().clone());
                    } else {
                        new_body.push(bc.clone());
                    }
                }
            }

            // TODO : use mem::replace to avoid a lot of this extra work
            let tokens = self.fun_table
                .get(&fname)
                .unwrap()
                .as_user_fun()
                .tokens
                .clone();

            let locals = self.fun_table
                .get(&fname)
                .unwrap()
                .as_user_fun()
                .locals
                .clone();

            // replace the function with the new body
            self.fun_table.insert(
                fname.clone(),
                Fun::UserFun(BCUserFun::new(fname, new_body, tokens, locals)),
            );
        }
    }
}
