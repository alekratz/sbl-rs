mod vm;
mod builtins;
mod foreign;
mod val;
mod bc;

pub use self::vm::*;
pub use self::builtins::*;
pub use self::val::*;
pub use self::bc::*;

use syntax::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{self, Formatter, Display};

#[derive(Clone, Debug)]
pub struct UserFun {
    pub name: String,
    pub body: BcBody,
    pub tokens: Tokens,
    pub contains_bake: bool,
}

impl UserFun {
    pub fn new(name: String, body: BcBody, tokens: Tokens) -> Self {
        let contains_bake = tokens.contains_bake_token();
        UserFun {
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
}

#[derive(EnumIsA)]
pub enum Fun {
    UserFun(Rc<UserFun>),
    ForeignFun(ForeignFn),
    BuiltinFun(&'static BuiltinFun),
}

impl Fun {
    pub fn user_fun(&self) -> &UserFun {
        if let &Fun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("Fun::user_fun() called on non-UserFun item")
        }
    }
}

pub type FunTable = HashMap<String, Fun>;
pub type FunRcTable = HashMap<String, Rc<Fun>>;
