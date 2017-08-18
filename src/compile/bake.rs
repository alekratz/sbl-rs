use compile::{Compile, CompileIRBlock};
use common::*;
use errors::*;
use syntax::*;
use vm::*;

use itertools::interleave;
use std::collections::HashMap;
use std::rc::Rc;

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
    fn compile(self) -> Result<Self::Out> {
        // make sure that no functions being called contain bakes themselves
        for fun in self.fun_table.values().filter(|f| f.is_user_fun()) {
            let mut fun = fun.as_user_fun().clone();
            // gets a list of all the bake blocks for this function
            let bake_blocks = fun.body
                .iter()
                .filter_map(|b| if b.bc_type == BCType::Bake { b.val.clone() } else { None })
                .filter_map(|v| if let Val::BakeBlock(b) = v { Some(b) } else { None })
                .collect::<Vec<_>>();

            // what the below does is gets the list of function calls in all bake statements that
            // contain function calls to other bake statements
            let illegal_calls = bake_blocks
                .iter()
                .flat_map(|b| &b.block)
                .filter_map(|stmt| if let &Stmt::Stack(ref stmt) = stmt { Some(&stmt.stack_actions) } else { None })
                .flat_map(|actions| actions)
                .filter_map(|a| if let &StackAction::Push(ref i) = a { Some(i) } else { None })
                .filter_map(|i| if let ItemType::Ident(ref fname) = i.item_type { self.fun_table.get(fname) } else { None })
                .filter_map(|fun| if let &Fun::UserFun(ref fun) = fun { if fun.contains_bake { Some(&fun.name) } else { None } } else { None })
                .collect::<Vec<_>>();

            // Check if there's any recursive bake blocks
            if !illegal_calls.is_empty() {
                return (Err(format!("attempted to call `{}`, which contains a `bake` statement itself", illegal_calls[0]).into()) as Result<_>)
                    .chain_err(|| format!("in `bake` statement in function `{}`", fun.name))
                    .chain_err(|| "recursive `bake` statements are not allowed");
            }

            let baked = bake_blocks
                .into_iter()
                .map(|block| {
                    let compiled = {
                        let compile_block = CompileIRBlock::new(&self.fun_table, &block, 0);
                        compile_block.compile().map(|mut b| {
                            b.push(BC::ret(block.tokens().into()));
                            b
                        })
                    };
                    (compiled, block)
                })
                .collect::<Vec<_>>();

            // Catch any compile errors in the bake blocks
            if baked.iter().any(|&(ref r, _)| r.is_err()) {
                // NOTE: do not try to refactor this.
                // * Errs cannot be cloned
                // * `if let Some(e) = ...` will make 'e' a ref, and you need an owned error.
                // TRUST ME
                baked.into_iter().find(|&(ref r, _)| r.is_err()).unwrap().0?;
                unreachable!();
            }

            // Take all compiled blocks, and run them through the VM, producing a list of "push"
            // instructions for the last values on the stack
            let baked_compiled = baked
                .into_iter()
                .map(|(compiled, block)| (compiled.unwrap(), block))
                .map(|(compiled, block)| {
                    let baked_name = format!("< baked block defined in {} >", block.range());
                    let block_tokens: Tokens = block.tokens().into();
                    let baked_fun =
                        UserFun::new(baked_name.clone(), compiled, block_tokens.clone());
                    let mut baked_table = self.fun_table.clone();
                    baked_table.insert(baked_name.clone(), Fun::UserFun(Rc::new(baked_fun)));
                    let mut vm = VM::new(baked_table);
                    match vm.invoke(&baked_name) {
                        Ok(_) => {
                            let vm_state: State = vm.into();
                            Ok(vm_state.stack
                                    .into_iter()
                                    .map(|val| BC::push(block_tokens.clone(), val))
                                    .collect::<Vec<_>>())
                        }
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();

            // Catch any VM errors that may have occurred and return them
            if baked_compiled.iter().any(Result::is_err) {
                baked_compiled.into_iter().find(Result::is_err).unwrap()?;
                unreachable!();
            }

            let baked_compiled = baked_compiled
                .into_iter()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            let body = fun.body
                .split(|b| b.bc_type == BCType::Bake)
                .map(|b| b.to_vec())
                .collect::<Vec<_>>();
            assert_eq!(body.len() - 1, baked_compiled.len());

            fun.body = interleave(body, baked_compiled)
                .into_iter()
                .flat_map(id)
                .collect::<Vec<_>>();
            boring_table.insert(fun.name.clone(), Some(Fun::UserFun(Rc::new(fun))));
        }
        Ok(
            boring_table
                .into_iter()
                .map(|(k, v)| (k, v.unwrap()))
                .collect(),
        )
    }
}
