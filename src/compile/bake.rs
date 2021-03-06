use prelude::*;
use petgraph::algo::toposort;
use std::cell::RefCell;

pub struct BakeIRFunTable {
    bake_graph: CallGraph,
    ir_fun_table: IRFunTable,
    bc_fun_table: BCFunTable,
    vm: RefCell<VM>,
}

impl BakeIRFunTable {
    pub fn new(bake_graph: CallGraph, ir_fun_table: IRFunTable, bc_fun_table: BCFunTable) -> Self {
        BakeIRFunTable {
            bake_graph,
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
                let locals: Vec<String> = ir_body.iter()
                    .filter_map(|ir| if ir.ir_type == IRType::Pop && ir.val.as_ref().map(|v| v.is_ident()).unwrap_or(false) {
                        ir.val.as_ref().map(|v| v.as_ident().clone())
                    } else {
                        None
                    })
                    .collect();
                let mut compiled = self.compile_ir_body(ir_body)?;
                compiled.push(BC::ret(tokens.clone()));
                let mut vm = self.vm.borrow_mut();
                vm.clear_state();
                vm.inject_user_fun(BCUserFun::new(format!("<bake block at {}>", tokens.range()), compiled, tokens.clone(), locals))?;
                let state: State = vm.clone()
                    .into();
                Ok(state.stack
                    .into_iter()
                    .map(|v| BC::push(tokens.clone(), BCVal::PushAll(vec![v])))
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

impl Compile for BakeIRFunTable {
    type Out = BCFunTable;
    fn compile(self) -> Result<Self::Out> {
        // Build the boring table
        let mut dep_order = match toposort(&self.bake_graph, None) {
            Err(cycle) => {
                return Err(format!("bake call cycle detected in function `{}`", &self.bake_graph[cycle.node_id()]).into());
            }
            Ok(v) => v,
        };

        dep_order.reverse();
        let baked_funs = dep_order.into_iter()
            .map(|fun_index| {
                let fname = &self.bake_graph[fun_index];
                let fun = self.ir_fun_table[fname].as_user_fun();
                let locals: Vec<String> = fun.body
                    .iter()
                    .filter_map(|ir| if ir.ir_type == IRType::Pop && ir.val.as_ref().map(|v| v.is_ident()).unwrap_or(false) {
                        ir.val.as_ref().map(|v| v.as_ident().clone())
                    } else {
                        None
                    })
                    .collect();
                let body = match self.compile_ir_body(fun.body.clone()) {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                };
                let userfun = Fun::UserFun(BCUserFun::new(fname.to_string(), body, fun.tokens.clone(), locals));
                let mut vm = self.vm.borrow_mut();
                vm.add_fun(fname.to_string(), userfun.clone());
                Ok(userfun)
            }).collect::<Vec<_>>();
        if baked_funs.iter().any(Result::is_err) {
            // TODO : error lists, so we can get all errors with baked functions and not just the
            // first one
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
