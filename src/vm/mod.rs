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
    pub fn as_user_fun(&self) -> &UserFun {
        if let &Fun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("Fun::as_user_fun() called on non-UserFun item")
        }
    }
}

impl Clone for Fun {
    fn clone(&self) -> Self {
        match self {
            &Fun::UserFun(ref fun) => Fun::UserFun(fun.clone()),
            &Fun::ForeignFun(ref fun) => Fun::ForeignFun(fun.clone()),
            &Fun::BuiltinFun(fun) => Fun::BuiltinFun(fun),
        }
    }
}

/*
// see https://github.com/rust-lang/rust/issues/26264 as to why this doesn't work :|
pub enum VmFun<F: 'static> where F: Fn(&mut State) -> Result<()> {
    UserFun(Rc<UserFun>),
    ForeignFun(ForeignFn),
    BuiltinFun(F),
}

impl<F: 'static> VmFun<F> where F: Fn(&mut State) -> Result<()> {
    pub fn is_user_fun(&self) -> bool {
        matches!(self, &Fun::UserFun(_))
    }

    pub fn user_fun(&self) -> &UserFun {
        if let &Fun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("Fun::user_fun() called on non-UserFun item")
        }
    }
}

pub type Fun = VmFun<&'static BuiltinFun>;
*/

pub type FunTable = HashMap<String, Fun>;
pub type FunRcTable = HashMap<String, Rc<Fun>>;
