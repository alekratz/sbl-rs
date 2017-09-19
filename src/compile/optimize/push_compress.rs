use prelude::*;

/// An optimizer which compresses strings of PUSH instructions into a single instruction.
pub struct PushCompress {
    fun_table: BCFunTable,
}

impl PushCompress {
    pub fn new(fun_table: BCFunTable) -> Self {
        PushCompress { fun_table }
    }
}

impl Optimize for PushCompress {
    type Out = BCFunTable;

    fn optimize(self) -> Self::Out {
        self.fun_table
            .into_iter()
            .map(|(name, mut fun)| {
                if let Fun::UserFun(ref mut fun) = fun {
                    fun.compress_pushes();
                }
                (name, fun)
            })
            .collect()
    }
}


