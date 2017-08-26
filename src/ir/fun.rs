use ir::*;
use syntax::*;
use internal::*;
use std::collections::HashMap;
use std::rc::Rc;

pub type IRBody = Vec<IR>;

#[derive(Clone, Debug)]
pub struct IRUserFun {
    pub name: String,
    pub body: IRBody,
    pub tokens: Tokens,
    pub contains_bake: bool,
}

impl IRUserFun {
    pub fn new(name: String, body: IRBody, tokens: Tokens) -> Self {
        let contains_bake = tokens.contains_bake_token();
        IRUserFun {
            name,
            body,
            tokens,
            contains_bake,
        }
    }

    pub fn dump(&self) {
        let mut addr = 0;
        for bc in &self.body {
            eprintln!(
                "{:06} {:6} {}",
                addr,
                &bc.ir_type.to_string(),
                if let Some(ref payload) = bc.val {
                    format!("{:?}", payload)
                } else {
                    format!("")
                }
            );
            addr += 1;
        }
    }
}

impl UserFun for IRUserFun {
    type InstructionT = IR;

    fn name(&self) -> &str { &self.name }
    fn body(&self) -> &[Self::InstructionT] { &self.body }
    fn tokens(&self) -> &[Rc<Token>] { &self.tokens }
}

pub type IRFun = Fun<IRUserFun>;

impl IRFun { }

pub type IRFunTable = HashMap<String, IRFun>;
