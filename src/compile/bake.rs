use compile::*;
use common::*;
use errors::*;
use syntax::*;
use vm::*;
use ir::*;

use itertools::interleave;
use petgraph::Direction;
use petgraph::algo::is_cyclic_directed;
use petgraph::visit::Dfs;
use std::collections::HashMap;
use std::rc::Rc;

pub struct BakeIR {
    fun_table: IRFunTable,
}

impl BakeIR {
    pub fn new(fun_table: IRFunTable) -> Self {
        BakeIR { fun_table }
    }
}

impl Compile for BakeIR {
    type Out = IRFunTable;
    fn compile(self) -> Result<Self::Out> {
        let bake_graph = build_bake_call_graph(&self.fun_table);
        if is_cyclic_directed(&bake_graph) {
            return Err("there were one or more cycles detected in bake blocks".into());
        }

        // find all leaves and compile them first
        let leaves = bake_graph.node_indices()
            .filter(|node| bake_graph.neighbors_directed(*node, Direction::Incoming).count() == 0)
            .collect::<Vec<_>>();
        
        println!("found the following leaves in the bake statement graph:");
        for l in leaves {
            println!("    {}", &bake_graph[l]);
        }

        let boring_table = self.fun_table
            .clone()
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<HashMap<_, _>>();
        // make sure that no functions being called contain bakes themselves
        for fun in self.fun_table.values().filter(|f| f.is_user_fun()) {
            let mut fun = fun.as_user_fun().clone();
            // gets a list of all the bake blocks for this function
            let bake_blocks = fun.body
                .iter()
                .filter_map(|b| if b.ir_type == IRType::Bake { b.val.clone() } else { None })
                .filter_map(|v| if let IRVal::BakeBlock(b) = v { Some(b) } else { None })
                .collect::<Vec<_>>();

            let baked = bake_blocks
                .into_iter()
                .map(|block| {
                    let compiled = {
                        let compile_block = CompileIRBlock::new(&boring_table, &block, 0);
                        compile_block.compile().map(|mut b| {
                            b.push(IR::ret(block.tokens().into()));
                            b
                        })
                    };
                    (compiled, block)
                })
                .collect::<Vec<_>>();

            /*
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
                        IRUserFun::new(baked_name.clone(), compiled, block_tokens.clone());
                    let mut baked_table = self.fun_table.clone();
                    baked_table.insert(baked_name.clone(), IRFun::UserFun(baked_fun));
                    let mut vm = VM::new(baked_table);
                    match vm.invoke(&baked_name) {
                        Ok(_) => {
                            let vm_state: State = vm.into();
                            Ok(vm_state.stack
                                    .into_iter()
                                    .map(|val| IR::push(block_tokens.clone(), val.into()))
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
                .split(|b| b.ir_type == IRType::Bake)
                .map(|b| b.to_vec())
                .collect::<Vec<_>>();
            assert_eq!(body.len() - 1, baked_compiled.len());

            fun.body = interleave(body, baked_compiled)
                .into_iter()
                .flat_map(id)
                .collect::<Vec<_>>();
            boring_table.insert(fun.name.clone(), Some(IRFun::UserFun(fun)));
            */
        }
        Ok(
            boring_table
                .into_iter()
                .map(|(k, v)| (k, v.unwrap()))
                .collect(),
        )
    }
}
