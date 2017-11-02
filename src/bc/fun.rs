use prelude::*;
//use itertools::Itertools;
use std::collections::BTreeMap;
use std::rc::Rc;

pub type BCFunTable = BTreeMap<String, BCFun>;
pub type BCFunRcTable = BTreeMap<String, Rc<BCFun>>;

#[derive(Clone, Debug)]
pub struct BCUserFun {
    pub name: String,
    pub body: BCBody,
    pub tokens: Tokens,
    pub locals: Vec<String>,
}

impl BCUserFun {
    pub fn new(name: String, body: BCBody, tokens: Tokens, locals: Vec<String>) -> Self {
        BCUserFun {
            name,
            body,
            tokens,
            locals,
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
                "{:06} {:8}{}{}",
                addr,
                &bc.bc_type.to_string(),
                if let Some(ref payload) = bc.val {
                    format!(" {:?}", payload)
                } else {
                    format!("")
                },
                if let Some(ref target) = bc.target {
                    format!(" -> {:?}", target)
                } else {
                    String::new()
                }
            );
            addr += 1;
        }
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
        // get the list of local variables used in this function
        let mut locals: Vec<String> = other.body
            .iter()
            .filter_map(|ir| if ir.ir_type == IRType::Pop && ir.val.as_ref().map(|v| v.is_ident()).unwrap_or(false) {
                ir.val.as_ref().map(|v| v.as_ident().clone())
            } else {
                None
            })
            .collect();
        // sort them; this may make it faster in some contexts
        locals.sort();

        BCUserFun {
            name: other.name,
            body: other.body
                .into_iter()
                .map(BC::from)
                //            this ugly conditional checks to see if we need to replace a
                //            load/store with a number
                .map(|mut bc| if (bc.bc_type == BCType::Pop || bc.bc_type == BCType::Load) && bc.val.as_ref().map(|v| v.is_ident()).unwrap_or(false) {
                    let position = {
                        let name = bc.val
                            .as_ref()
                            .unwrap()
                            .as_ident();
                        locals.iter()
                            .position(|n| n == name)
                            .expect(&format!("could not find local variable {}", name))
                    };
                    bc.val = Some(BCVal::Address(position));
                    bc
                } else { bc })
                .collect(),
            tokens: other.tokens,
            locals,
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
