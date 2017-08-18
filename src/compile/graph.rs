use syntax::*;
use ir::*;
use petgraph::Graph;
use std::collections::{HashMap, HashSet};

pub fn build_call_graph(fun_table: &IRFunTable) -> Graph<&str, usize> {
    // build all of the funtable nodes
    let mut fun_graph = Graph::new();
    let mut node_table = HashMap::new();
    for fun in fun_table.keys() {
        let node = fun_graph.add_node(fun.as_str());
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
                    fun_graph.add_edge(*node, *callee, 1);
                }
            }
        }
    }
    fun_graph
}

pub fn build_bake_call_graph(fun_table: &IRFunTable) -> Graph<&str, usize> {
    /// Utility function that recursively grabs all function calls from a bake statement.
    fn get_all_bake_calls<'a>(block: &'a Block, fun_table: &'a IRFunTable) -> HashSet<&'a str> {
        let mut funs = hashset!{};
        for stmt in &block.block {
            match stmt {
                &Stmt::Stack(StackStmt { stack_actions: ref actions, tokens: _ }) => {
                    let filter = actions.iter()
                        .filter_map(|a| if let &StackAction::Push(Item { item_type: ItemType::Ident(ref name), tokens: _ }) = a {
                            Some(name.as_str())
                        } else {
                            None
                        });
                    for name in filter {
                        if fun_table.contains_key(name) {
                            funs.insert(name);
                        }
                    }
                },
                &Stmt::Bake(ref bake) => {
                    funs = funs.union(&get_all_bake_calls(&bake.block, fun_table))
                        .cloned()
                        .collect();
                },
                _ => { },
            }
        }
        funs
    }

    // build all of the funtable nodes
    let mut fun_graph = Graph::new();
    let mut node_table = HashMap::new();

    for (fname, fun) in fun_table.iter() {
        if fun.is_user_fun() && fun.as_user_fun().contains_bake {
            let node = fun_graph.add_node(fname.as_str());
            node_table.insert(fname.to_string(), node);
        }
    } 

    // hook up function calls
    for (fname, node) in &node_table {
        let fun = fun_table.get(fname).unwrap();
        if let &IRFun::UserFun(ref fun) = fun {
            for ir in &fun.body {
                if ir.ir_type == IRType::Bake {
                    let (_, block) = ir.val
                        .as_ref()
                        .unwrap()
                        .as_bake_block();
                    let funcalls = get_all_bake_calls(block, fun_table);
                    for fname in funcalls {
                        if let Some(callee) = node_table.get(fname) {
                            fun_graph.add_edge(*node, *callee, 1);
                        }
                    }
                }
            }
        }
    }
    fun_graph
}
