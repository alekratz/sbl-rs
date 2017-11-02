mod inline;

use prelude::*;
use self::inline::*;
use std::mem;
use std::collections::{HashMap, BTreeMap};

/// A general optimization trait.
pub trait Optimize {
    type Out;

    fn optimize(self) -> Self::Out;
}

/// An optimize map maps each user function using a given `FnMut(&mut BCUserFun)` closure.
struct OptimizeMap<M> where M: FnMut(&mut BCUserFun) {
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
        const STORE                 = 1 << 1;
        const PUSH_COMPRESS         = 1 << 2;
        const ABSOLUTE_JUMPS        = 1 << 3;
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
                Optimizations::PUSH_COMPRESS => BCUserFun::optimize_push_compress as fn(&mut BCUserFun),
                Optimizations::ABSOLUTE_JUMPS => BCUserFun::optimize_jumps,
                Optimizations::STORE => BCUserFun::optimize_store,
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
            // STORE converts a PUSH of a const immediately followed by a POP to a
            // STORE instruction
            Optimizations::STORE,
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

/*
 * BCUserFun functions for optimization specifically.
 */

impl BCUserFun {
    /// Compresses all adjacent push statements to one statement.
    ///
    /// This is used exclusively by the optimizer.
    pub(in compile::optimize) fn optimize_push_compress(&mut self) {
        let body = self.body
            .clone()
            .into_iter();
        // TODO : move this to using mem::replace
        let mut last_was_push = false;
        self.body = body.fold(vec![], |mut body, instr| {
            if instr.bc_type == BCType::Push {
                if last_was_push {
                    let last_part = body.last_mut()
                        .unwrap();
                    last_part.val
                        .as_mut()
                        .unwrap()
                        .append(&mut instr.val.unwrap());
                } else {
                    body.push(instr);
                    last_was_push = true;
                }
            } else {
                body.push(instr);
                last_was_push = false;
            }
            body
        });
    }

    /// Converts all SYM_JUMP* instructions into JMP* instructions. This speeds up program
    /// execution by not having to do a label lookup every time a jump is done.
    ///
    /// This is used exclusively by the optimizer.
    pub(in compile::optimize) fn optimize_jumps(&mut self) {
        // Create a label table
        let labels: BTreeMap<i64, usize> = self.body
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut labels, (addr, instr)| {
                if instr.bc_type == BCType::Label {
                    let lblcount = labels.len();
                    labels.insert(*instr.val.as_ref().unwrap().as_int(), addr - lblcount);
                }
                labels
            });
        // Replace symbolic jumps with absolute jumps, and remove the labels as well
        self.body = mem::replace(&mut self.body, Vec::new())
            .into_iter()
            .filter(|instr| instr.bc_type != BCType::Label)
            .map(|instr| if instr.bc_type == BCType::SymJmp { BC::jmp(instr.tokens, labels[instr.val.unwrap().as_int()].into()) }
                 else if instr.bc_type == BCType::SymJmpZ { BC::jmpz(instr.tokens, labels[instr.val.unwrap().as_int()].into()) }
                 else { instr })
            .collect()
    }

    /// Converts a push followed immediately by a pop into a single "store" instruction.
    /// This cuts down on wasted VM cycles and memory transactions.
    ///
    /// This is used exclusively by the optimizer.
    pub(in compile::optimize) fn optimize_store(&mut self) {
        let mut last_was_push = false;
        self.body = mem::replace(&mut self.body, vec![])
            .into_iter()
            .fold(vec![], |mut body, instr| {
                if instr.bc_type == BCType::Pop && last_was_push {
                    // pop the last item; ensure that it's a push
                    let last = body.pop().unwrap();
                    assert!(last.bc_type == BCType::Push);
                    let (mut tokens, mut val) = (last.tokens, last.val);
                    if let Some(BCVal::PushAll(mut pushall)) = val {
                        assert!(pushall.len() == 1, "Pushall value length was not 1; are STORE optimizations being done before PUSH_COMPRESS?");
                        val = pushall.pop();
                    }
                    tokens.extend_from_slice(instr.tokens.as_slice());
                    // TODO : change POP to use 'target' instead of 'val'
                    let target = instr.val;
                    assert_matches!(target, Some(BCVal::Address(_)));
                    body.push(BC { bc_type: BCType::Store, tokens, target, val });
                }
                else {
                    last_was_push = instr.bc_type == BCType::Push;
                    body.push(instr);
                }
                body
            });
    }
}
