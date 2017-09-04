use prelude::*;
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
        if let &Fun::UserFun(ref fun) = fun {
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
    fn get_all_bake_calls<'a>(body: &'a [IR], fun_table: &'a IRFunTable, baked_funs: &mut HashSet<&'a str>, called_funs: &mut HashSet<&'a str>) -> Result<()> {
        for ref ir in body {

            match &ir.ir_type {
                &IRType::Call => {
                    let name = ir.val
                        .as_ref()
                        .unwrap()
                        .as_ident();
                    if baked_funs.contains(name.as_str()) {
                        return Err(format!("which calls `{}`", name).into());
                    }
                    let user_fun = fun_table.get(name);
                    if user_fun.map(Fun::is_user_fun).unwrap_or(false) {
                        let user_fun = user_fun.unwrap()
                            .as_user_fun();
                        if user_fun.contains_bake {
                            baked_funs.insert(name.as_str());
                        }
                        if !called_funs.contains(name.as_str()) {
                            called_funs.insert(name.as_str());
                            get_all_bake_calls(&user_fun.body, fun_table, baked_funs, called_funs)
                                .chain_err(|| format!("which calls `{}` (in {})", name, user_fun.tokens.range()))?;
                        }
                    }
                }
                &IRType::Bake => {
                    let bake_body = ir.val
                        .as_ref()
                        .unwrap()
                        .as_bake_block();
                    get_all_bake_calls(bake_body, fun_table, baked_funs, called_funs)?;
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
        if let &Fun::UserFun(ref fun) = fun {
            for ir in &fun.body {
                if ir.ir_type == IRType::Bake {
                    let body = ir.val
                        .as_ref()
                        .unwrap()
                        .as_bake_block();
                    let mut baked_funs: HashSet<&str> = hashset!();
                    let mut called_funs = hashset!(fun.name.as_str());
                    get_all_bake_calls(&body, fun_table, &mut baked_funs, &mut called_funs)
                        .chain_err(|| format!("cycle detected in `{}` (in {})", fun.name, fun.tokens.range()))?;
                    for fname in baked_funs {
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
