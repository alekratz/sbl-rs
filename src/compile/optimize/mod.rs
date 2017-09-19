mod inline;

use prelude::*;
use self::inline::*;

use std::mem;
use std::collections::HashMap;

/// A general optimization trait.
pub trait Optimize {
    type Out;

    fn optimize(self) -> Self::Out;
}

/// An optimize map maps each user function using a given `FnMut(&mut BCUserFun)` closure.
pub struct OptimizeMap<M> where M: FnMut(&mut BCUserFun) {
    pub fun_table: BCFunTable,
    map_fn: M,
}

impl<M> OptimizeMap<M> where M: FnMut(&mut BCUserFun) {
    pub fn new(fun_table: BCFunTable, map_fn: M) -> Self {
        OptimizeMap {
            fun_table,
            map_fn,
        }
    }
}

impl<M> Optimize for OptimizeMap<M> where M: FnMut(&mut BCUserFun) {
    type Out = BCFunTable;
    fn optimize(self) -> Self::Out {
        let (mut map_fn, fun_table) = (self.map_fn, self.fun_table);
        fun_table.into_iter()
            .map(|(name, mut fun)| {
                if let Fun::UserFun(ref mut fun) = fun {
                    map_fn(fun);
                }
                (name, fun)
            })
            .collect()
    }
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
        // map of optimize map functions
        lazy_static! {
            static ref OPTIMAP: HashMap<Optimizations, fn(&mut BCUserFun)> = hashmap! {
                Optimizations::PUSH_COMPRESS => BCUserFun::compress_pushes as fn(&mut BCUserFun),
                Optimizations::ABSOLUTE_JUMPS => BCUserFun::apply_absolute_jumps,
            };
        }

        // move the fun_table out
        let fun_table = mem::replace(&mut self.fun_table, BCFunTable::new());
        
        // mini optimizer factory
        match opt {
            Optimizations::INLINE => {
                // inline optimization, which needs more than a map
                let opty = Inline::new(fun_table);
                self.fun_table = opty.optimize();
            }
            _ => {
                // map-based optimizations
                let opty = OptimizeMap::new(fun_table, OPTIMAP[&opt]);
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
            // INLINE can happen anywhere. Doing it before PUSH_COMPRESS is a good idea because
            // then any inlined PUSH instructions will get compressed later on.
            Optimizations::INLINE,
            // PUSH_COMPRESS should happen near the end, because some pushes may get removed or
            // added to fix this up.
            Optimizations::PUSH_COMPRESS,
            // ABSOLUTE_JUMPS happens last, because modifying the order of absolute jumps will mess
            // up the program.
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
