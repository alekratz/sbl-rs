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
use ir::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{self, Formatter, Display};

#[derive(Clone, Debug)]
pub struct UserFun {
    pub name: String,
    pub body: BCBody,
    pub tokens: Tokens,
}

impl UserFun {
    pub fn new(name: String, body: BCBody, tokens: Tokens) -> Self {
        UserFun {
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
}

impl From<IRUserFun> for UserFun {
    fn from(other: IRUserFun) -> Self {
        UserFun {
            name: other.name,
            body: other.body
                .into_iter()
                .map(IR::into)
                .collect(),
            tokens: other.tokens,
        }
    }
}

#[derive(EnumIsA)]
pub enum Fun {
    UserFun(UserFun),
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

impl From<IRFun> for Fun {
    fn from(other: IRFun) -> Self {
        match other {
            IRFun::UserFun(u) => Fun::UserFun(u.into()),
            IRFun::ForeignFun(f) => Fun::ForeignFun(f),
            IRFun::BuiltinFun(b) => Fun::BuiltinFun(b),
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
