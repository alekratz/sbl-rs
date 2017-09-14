use prelude::*;
//use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;

pub type BCFunTable = HashMap<String, BCFun>;
pub type BCFunRcTable = HashMap<String, Rc<BCFun>>;

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

    pub fn dump(&self) {
        let mut addr = 0;
        for bc in &self.body {
            eprintln!(
                "{:06} {:6} {}",
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
