use syntax::*;
use vm::*;

pub struct Bake<'ast, 'ft> {
    ast: &'ast mut AST,
    fun_table: &'ft mut FunTable,
}

impl<'ast, 'ft> Bake<'ast, 'ft> {
    pub fn new(ast: &'ast mut AST, fun_table: &'ft mut FunTable) -> Self {
        Bake { ast, fun_table }
    }

    pub fn bake(mut self) {
        unimplemented!()
    }
}
