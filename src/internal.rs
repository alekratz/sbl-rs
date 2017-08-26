use vm::*;
use syntax::*;
use std::fmt::{Debug, Formatter, self};
use std::rc::Rc;

pub trait UserFun {
    type InstructionT;

    fn name(&self) -> &str;
    fn body(&self) -> &[Self::InstructionT];
    fn tokens(&self) -> &[Rc<Token>];
}

/// A marker trait that defines an instruction type.
pub trait Instruction {}

pub enum Fun<U: UserFun> {
    UserFun(U),
    ForeignFun(ForeignFun),
    BuiltinFun(&'static BuiltinFun),
}


impl<U: UserFun> Fun<U> {
    pub fn as_user_fun(&self) -> &U {
        if let &Fun::UserFun(ref fun) = self {
            fun
        } else {
            panic!("Fun::as_user_fun() called on non-BCU item")
        }
    }

    pub fn is_user_fun(&self) -> bool {
        matches!(self, &Fun::UserFun(_))
    }
}

impl<U: UserFun + Clone> Clone for Fun<U> {
    fn clone(&self) -> Self {
        match self {
            &Fun::UserFun(ref fun) => Fun::UserFun(fun.clone()),
            &Fun::ForeignFun(ref fun) => Fun::ForeignFun(fun.clone()),
            &Fun::BuiltinFun(fun) => Fun::BuiltinFun(fun),
        }
    }
}

impl<U: UserFun + Debug> Debug for Fun<U> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", match self {
            &Fun::UserFun(ref fun) => format!("{:?}", fun),
            &Fun::ForeignFun(ref fun) => format!("{:?}", fun),
            &Fun::BuiltinFun(fun) => format!("{:?}", fun as *const _),
        })
    }
}

