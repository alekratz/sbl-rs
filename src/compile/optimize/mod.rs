mod absolute_jumps;
mod inline;
mod push_compress;

use prelude::*;
use self::absolute_jumps::*;
use self::inline::*;
use self::push_compress::*;

use std::mem;

/// A general optimization trait.
pub trait Optimize {
    type Out;

    fn optimize(self) -> Self::Out;
}

/// These are the optimization flags which can enable a set of optimizations.
bitflags! {
    pub struct Optimizations: u32 {
        const INLINE                = 1 << 0;
        const PUSH_COMPRESS         = 1 << 1;
        const ABSOLUTE_JUMPS        = 1 << 2;
    }
}

impl Default for Optimizations {
    fn default() -> Self {
        Self::all()
    }
}

/// An optimization pipeline. This passes the bytecode from one optimizer to the next in the
/// correct order, with the option to turn off specific optimizations.
pub struct OptimizePipeline {
    pub flags: Optimizations,
    fun_table: BCFunTable,
}

impl OptimizePipeline {
    /// Creates a new optimization pipeline with the default optimizations enabled.
    pub fn new(fun_table: BCFunTable) -> Self {
        OptimizePipeline { flags: Optimizations::default(), fun_table }
    }

    /// Applies the given optimization.
    fn apply(&mut self, opt: Optimizations) {
        // move the fun_table out
        let fun_table = mem::replace(&mut self.fun_table, BCFunTable::new());
        // mini optimizer factory
        match opt {
            Optimizations::INLINE => {
                let opty = Inline::new(fun_table);
                self.fun_table = opty.optimize();
            }
            Optimizations::PUSH_COMPRESS => {
                let opty = PushCompress::new(fun_table);
                self.fun_table = opty.optimize();
            }
            Optimizations::ABSOLUTE_JUMPS => {
                let opty = AbsoluteJumps::new(fun_table);
                self.fun_table = opty.optimize();
            }
            _ => panic!("unknown optimization flag: {}", opt.bits)
        }
    }
}

impl Optimize for OptimizePipeline {
    type Out = BCFunTable;

    fn optimize(mut self) -> Self::Out {
        // this is the absolute order that optimizations must be executed in
        let optimization_order = vec![
            Optimizations::INLINE,
            Optimizations::PUSH_COMPRESS,
            Optimizations::ABSOLUTE_JUMPS,
        ];

        for opt in optimization_order {
            if self.flags.contains(opt) {
                self.apply(opt);
            }
        }
        self.fun_table
    }

}
