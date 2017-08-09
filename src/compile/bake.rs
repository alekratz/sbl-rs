use compile::Compile;
use errors::*;
use syntax::*;
use vm::*;

use std::collections::HashMap;

pub struct BakeBytes {
    fun_table: FunTable,
}

impl BakeBytes {
    pub fn new(fun_table: FunTable) -> Self {
        BakeBytes { fun_table }
    }
}

impl Compile for BakeBytes {
    type Out = FunTable;
    fn compile(mut self) -> Result<Self::Out> {
        let boring_table = self.fun_table
            .clone()
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<HashMap<_, _>>();
        // make sure that no functions being called contain bakes themselves
        for fun in self.fun_table.values().filter(|f| f.is_user_fun()) {
            let fun = fun.user_fun();
            // gets a list of all the bake blocks for this function
            let bake_blocks = fun.body
                .iter()
                .filter_map(|b| if b.bc_type == BcType::Bake {
                    b.val.clone()
                } else {
                    None
                })
                .filter_map(|v| if let Val::BakeBlock(b) = v {
                    Some(b)
                } else {
                    None
                })
                .collect::<Vec<_>>();

            // what the below does is gets the list of function calls in all bake statements that
            // contain function calls to other bake statements
            let illegal_calls = bake_blocks
                .iter()
                .flat_map(|b| &b.block)
                .filter_map(|stmt| if let &Stmt::Stack(ref stmt) = stmt {
                    Some(&stmt.stack_actions)
                } else {
                    None
                })
                .flat_map(|actions| actions)
                .filter_map(|a| if let &StackAction::Push(ref i) = a {
                    Some(i)
                } else {
                    None
                })
                .filter_map(|i| if let ItemType::Ident(ref fname) = i.item_type {
                    self.fun_table.get(fname)
                } else {
                    None
                })
                .filter_map(|fun| if let &Fun::UserFun(ref fun) = fun {
                    if fun.contains_bake {
                        Some(&fun.name)
                    } else {
                        None
                    }
                } else {
                    None
                })
                .collect::<Vec<_>>();
            if !illegal_calls.is_empty() {
                return (Err(
                    format!(
                        "attempted to call `{}`, which contains a `bake` statement itself",
                        illegal_calls[0]
                    ).into(),
                ) as Result<_>)
                    .chain_err(|| format!("in `bake` statement in function `{}`", fun.name))
                    .chain_err(|| "recursive `bake` statements are not allowed");
            }

            // go through all bake blocks, and compile them
            for block in bake_blocks {
                // TODO
            }
        }
        unimplemented!()
    }
}
