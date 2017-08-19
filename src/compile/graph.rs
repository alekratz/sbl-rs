use syntax::*;
use ir::*;
use internal::*;
use errors::*;
use petgraph::Graph;
use std::collections::{HashMap, HashSet};

pub type CallGraph = Graph<String, ()>;

pub fn build_call_graph(fun_table: &IRFunTable) -> CallGraph {
    // build all of the funtable nodes
    let mut fun_graph = Graph::new();
    let mut node_table = HashMap::new();
    for fun in fun_table.keys() {
        let node = fun_graph.add_node(fun.to_string());
        node_table.insert(fun.to_string(), node);
    }

    // hook up function calls
    for (fname, node) in &node_table {
        let fun = fun_table.get(fname).unwrap();
        if let &IRFun::UserFun(ref fun) = fun {
            for ir in &fun.body {
                if ir.ir_type == IRType::Call {
                    // find the node and hook it up
                    let callee_name = ir.val.as_ref()
                        .unwrap()
                        .as_string();
                    let callee = node_table.get(callee_name)
                        .unwrap();
                    fun_graph.add_edge(*node, *callee, ());
                }
            }
        }
    }
    fun_graph
}

pub fn build_bake_call_graph(fun_table: &IRFunTable) -> Result<CallGraph> {
    /// Utility function that recursively grabs all function calls from a bake statement.
    fn get_all_bake_calls<'a>(body: &'a [IR], fun_table: &'a IRFunTable, funs: &mut HashSet<&'a str>) -> Result<()> {
        for ref ir in body {

            match &ir.ir_type {
                &IRType::Call => {
                    let name = ir.val
                        .as_ref()
                        .unwrap()
                        .as_ident();
                    if funs.contains(name.as_str()) {
                        return Err(format!("which calls `{}`", name).into());
                    }
                    if fun_table.get(name).map(IRFun::is_user_fun).unwrap_or(false) {
                        funs.insert(name.as_str());
                        get_all_bake_calls(&fun_table.get(name).unwrap().as_user_fun().body, fun_table, funs)
                            .chain_err(|| format!("which calls `{}`", name))?;
                    }
                }
                &IRType::Bake => {
                    let bake_body = ir.val
                        .as_ref()
                        .unwrap()
                        .as_bake_block();
                    get_all_bake_calls(bake_body, fun_table, funs)?;
                }
                _ => { }
            }
        }
        Ok(())
    }

    // build all of the funtable nodes
    let mut fun_graph = Graph::new();
    let mut node_table = HashMap::new();

    for (fname, fun) in fun_table.iter() {
        if fun.is_user_fun() && fun.as_user_fun().contains_bake {
            let node = fun_graph.add_node(fname.to_string());
            node_table.insert(fname.to_string(), node);
        }
    } 

    // hook up function calls
    for (fname, node) in &node_table {
        let fun = fun_table.get(fname).unwrap();
        if let &IRFun::UserFun(ref fun) = fun {
            for ir in &fun.body {
                if ir.ir_type == IRType::Bake {
                    let body = ir.val
                        .as_ref()
                        .unwrap()
                        .as_bake_block();
                    let mut funcalls = hashset!();
                    get_all_bake_calls(&body, fun_table, &mut funcalls)
                        .chain_err(|| format!("cycle detected in `{}`", fun.name))?;
                    for fname in funcalls {
                        if let Some(callee) = node_table.get(fname) {
                            fun_graph.add_edge(*node, *callee, ());
                        }
                    }
                }
            }
        }
    }
    Ok(fun_graph)
}
