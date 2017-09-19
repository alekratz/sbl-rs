use prelude::*;

/// An optimizer which gets the absolute addresses of jumps and applies those addresses, instead of
/// using symbolic jump labels. This also removes all labels from the bytecode.
pub struct AbsoluteJumps {
    fun_table: BCFunTable,
}

impl AbsoluteJumps {
    pub fn new(fun_table: BCFunTable) -> Self {
        AbsoluteJumps { fun_table }
    }
}

impl Optimize for AbsoluteJumps {
type Out = BCFunTable;

    fn optimize(self) -> Self::Out {
        self.fun_table
            .into_iter()
            .map(|(name, mut fun)| {
                if let Fun::UserFun(ref mut fun) = fun {
                    fun.apply_absolute_jumps();
                }
                (name, fun)
            })
            .collect()
    }
}
