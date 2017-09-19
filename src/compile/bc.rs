use prelude::*;

pub struct CompileBytes {
    fun_table: IRFunTable,
}

impl CompileBytes {
    pub fn new(fun_table: IRFunTable) -> Self {
        CompileBytes { fun_table }
    }
}

impl Compile for CompileBytes {
    type Out = BCFunTable;
    fn compile(self) -> Result<Self::Out> {
        let bake_graph = build_bake_call_graph(&self.fun_table)?;
        let (bc_funs, bake_funs): (IRFunTable, IRFunTable) = self.fun_table
            .into_iter()
            .partition(|&(_, ref v)| if let &Fun::UserFun(ref fun) = v {
                !fun.contains_bake
            } else {
                true
            });

        let bc_funs = bc_funs.into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<BCFunTable>();
        let bake_compile = BakeIRFunTable::new(bake_graph, bake_funs, bc_funs);
        bake_compile.compile()
    }
}

