use prelude::*;
//use itertools::Itertools;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::mem;

pub type BCFunTable = BTreeMap<String, BCFun>;
pub type BCFunRcTable = BTreeMap<String, Rc<BCFun>>;

#[derive(Clone, Debug)]
pub struct BCUserFun {
    pub name: String,
    pub body: BCBody,
    pub tokens: Tokens,
}

impl BCUserFun {
    pub fn new(name: String, body: BCBody, tokens: Tokens) -> Self {
        BCUserFun {
            name,
            body,
            tokens,
        }
    }

    /// Finds the address of the given label.
    pub fn get_label_address(&self, which: i64) -> usize {
        self.body
            .iter()
            .position(|bc| bc.bc_type == BCType::Label && *bc.val.as_ref().unwrap().as_int() == which)
            .expect(&format!("unknown label used: {}", which))
    }

    pub fn dump(&self) {
        let mut addr = 0;
        for bc in &self.body {
            eprintln!(
                "{:06} {:8} {}",
                addr,
                &bc.bc_type.to_string(),
                if let Some(ref payload) = bc.val {
                    format!("{:?}", payload)
                } else {
                    format!("")
                }
            );
            addr += 1;
        }
    }

    /// Compresses all adjacent push statements to one statement.
    pub fn compress_pushes(&mut self) {
        let body = self.body
            .clone()
            .into_iter();
        let mut last_was_push = false;
        self.body = body.fold(vec![], |mut body, instr| {
            if instr.bc_type == BCType::Push {
                if last_was_push {
                    let mut last_part = body.last_mut()
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

    pub fn apply_absolute_jumps(&mut self) {
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
}

impl UserFun for BCUserFun {
    type InstructionT = BC;

    fn name(&self) -> &str { self.name.as_str() }
    fn body(&self) -> &[Self::InstructionT] { self.body.as_slice() }
    fn tokens(&self) -> &[Rc<Token>] { self.tokens.as_slice() }
}

impl From<IRUserFun> for BCUserFun {
    fn from(other: IRUserFun) -> Self {
        BCUserFun {
            name: other.name,
            body: other.body
                .into_iter()
                .map(IR::into)
                .collect(),
            tokens: other.tokens,
        }
    }
}

pub type BCFun = Fun<BCUserFun>;

impl From<IRFun> for BCFun {
    fn from(other: IRFun) -> Self {
        match other {
            Fun::UserFun(u) => Fun::UserFun(u.into()),
            Fun::ForeignFun(f) => Fun::ForeignFun(f),
            Fun::BuiltinFun(b) => Fun::BuiltinFun(b),
        }
    }
}
