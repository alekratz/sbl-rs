use ir::*;
use bc::*;
use vm::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{Debug, Formatter, self};
use syntax::*;

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

#[derive(EnumIsA)]
pub enum BCFun {
    UserFun(BCUserFun),
    ForeignFun(ForeignFn),
    BuiltinFun(&'static BuiltinFun),
}

impl BCFun {
    pub fn as_user_fun(&self) -> &BCUserFun {
        if let &BCFun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("BCFun::as_user_fun() called on non-BCUserFun item")
        }
    }
}

impl Clone for BCFun {
    fn clone(&self) -> Self {
        match self {
            &BCFun::UserFun(ref fun) => BCFun::UserFun(fun.clone()),
            &BCFun::ForeignFun(ref fun) => BCFun::ForeignFun(fun.clone()),
            &BCFun::BuiltinFun(fun) => BCFun::BuiltinFun(fun),
        }
    }
}

impl From<IRFun> for BCFun {
    fn from(other: IRFun) -> Self {
        match other {
            IRFun::UserFun(u) => BCFun::UserFun(u.into()),
            IRFun::ForeignFun(f) => BCFun::ForeignFun(f),
            IRFun::BuiltinFun(b) => BCFun::BuiltinFun(b),
        }
    }
}

impl Debug for BCFun {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", match self {
            &BCFun::UserFun(ref fun) => format!("{:?}", fun),
            &BCFun::ForeignFun(ref fun) => format!("{:?}", fun),
            &BCFun::BuiltinFun(fun) => format!("{:?}", fun as *const _),
        })
    }
}

/*
// see https://github.com/rust-lang/rust/issues/26264 as to why this doesn't work :|
pub enum VmFun<F: 'static> where F: Fn(&mut State) -> Result<()> {
    BCUserFun(Rc<BCUserFun>),
    ForeignFun(ForeignFn),
    BuiltinFun(F),
}

impl<F: 'static> VmFun<F> where F: Fn(&mut State) -> Result<()> {
    pub fn is_user_fun(&self) -> bool {
        matches!(self, &BCFun::UserFun(_))
    }

    pub fn user_fun(&self) -> &BCUserFun {
        if let &BCFun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("BCFun::user_fun() called on non-BCUserFun item")
        }
    }
}

pub type BCFun = VmFun<&'static BuiltinFun>;
*/

