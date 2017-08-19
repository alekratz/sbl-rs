use syntax::*;
use ir::*;
use vm::*;
use compile::*;
use errors::*;
use internal::*;

use itertools::Itertools;
use petgraph::Direction;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::visit::Bfs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct BakeIRFun {
    ir_fun_table: IRFunTable,
    bc_fun_table: BCFunTable,
    vm: RefCell<VM>,
}

impl BakeIRFun {
    pub fn new(ir_fun_table: IRFunTable, bc_fun_table: BCFunTable) -> Self {
        BakeIRFun {
            ir_fun_table,
            bc_fun_table: bc_fun_table.clone(),
            vm: RefCell::new(VM::new(bc_fun_table)),
        }
    }

    fn compile_ir_body(&self, body: IRBody) -> Result<BCBody> {
        let bc_body_parts: Vec<Result<Vec<BC>>> = body.into_iter()
            .map(|ir| if ir.ir_type == IRType::Bake {
                let tokens = ir.tokens.clone();
                let ir_body = ir.val.unwrap()
                    .into_bake_block();
                let mut compiled = self.compile_ir_body(ir_body)?;
                compiled.push(BC::ret(tokens.clone()));
                let mut vm = self.vm.borrow_mut();
                vm.inject_user_fun(BCUserFun::new(format!("<bake block at {}>", tokens.range()), compiled, tokens.clone()))?;
                let state: State = vm.clone()
                    .into();
                Ok(state.stack
                    .into_iter()
                    .map(|v| BC::push(tokens.clone(), v))
                    .collect())
            } else {
                Ok(vec![ir.into()])
            }).collect();

        if bc_body_parts.iter().any(Result::is_err) {
            return bc_body_parts.into_iter().find(Result::is_err).unwrap();
        }
        Ok(bc_body_parts.into_iter()
            .flat_map(|r| r.unwrap())
            .collect())
    }
}

impl Compile for BakeIRFun {
    type Out = BCFunTable;
    fn compile(self) -> Result<Self::Out> {
        // Build the boring table
        /*
        let ir_boring_table = self.ir_fun_table
            .clone()
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<HashMap<_, _>>();
            */


        let bake_graph = build_bake_call_graph(&self.ir_fun_table);
        let mut dep_order = match toposort(&bake_graph, None) {
            Err(cycle) => {
                return Err(format!("bake call cycle detected in function `{}`", &bake_graph[cycle.node_id()]).into());
            }
            Ok(v) => v,
        };

        dep_order.reverse();
        let baked_funs = dep_order.into_iter()
            .map(|fun_index| {
                let fname = &bake_graph[fun_index];
                let fun = self.ir_fun_table[fname].as_user_fun();
                let body = match self.compile_ir_body(fun.body.clone()) {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                };
                let userfun = BCFun::UserFun(BCUserFun::new(fname.to_string(), body, fun.tokens.clone()));
                let mut vm = self.vm.borrow_mut();
                vm.add_fun(fname.to_string(), userfun.clone());
                Ok(userfun)
            }).collect::<Vec<_>>();
        if baked_funs.iter().any(Result::is_err) {
            Err(baked_funs.into_iter().find(Result::is_err)
                .unwrap()
                .unwrap_err())
        }
        else {
            
            let mut baked_funs = baked_funs.into_iter()
                .map(Result::unwrap)
                .map(|f| (f.as_user_fun().name.clone(), f))
                .collect::<BCFunTable>();
            baked_funs.extend(self.bc_fun_table);
            Ok(baked_funs)
        }
    }
}
