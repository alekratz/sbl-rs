use ir::*;
use vm::*;
use syntax::*;
use std::fmt::{Debug, Formatter, self};
use std::collections::HashMap;

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

#[derive(EnumIntoGetters, EnumIsA)]
pub enum IRFun {
    UserFun(IRUserFun),
    ForeignFun(ForeignFn),
    BuiltinFun(&'static BuiltinFun),
}

impl IRFun {
    pub fn as_user_fun(&self) -> &IRUserFun {
        if let &IRFun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("IRFun::as_user_fun() called on non-BCUserFun item")
        }
    }
}

impl Clone for IRFun {
    fn clone(&self) -> Self {
        match self {
            &IRFun::UserFun(ref fun) => IRFun::UserFun(fun.clone()),
            &IRFun::ForeignFun(ref fun) => IRFun::ForeignFun(fun.clone()),
            &IRFun::BuiltinFun(fun) => IRFun::BuiltinFun(fun),
        }
    }
}

impl Debug for IRFun {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", match self {
            &IRFun::UserFun(ref fun) => format!("{:?}", fun),
            &IRFun::ForeignFun(ref fun) => format!("{:?}", fun),
            &IRFun::BuiltinFun(fun) => format!("{:?}", fun as *const _),
        })
    }
}

pub type IRFunTable = HashMap<String, IRFun>;
