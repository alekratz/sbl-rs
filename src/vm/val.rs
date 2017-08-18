use errors::*;

use ir::*;
use std::cmp::Ordering;
use std::fmt::{self, Formatter, Display};

#[derive(EnumAsGetters, EnumIsA, PartialEq, Clone, Debug)]
pub enum Val {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<Val>),
    Nil,
}

impl Val {
    pub fn matches(&self, other: &Self) -> bool {
        match self {
            &Val::Int(_) => other.is_int(),
            &Val::Ident(_) => other.is_ident(),
            &Val::Char(_) => other.is_char(),
            &Val::String(_) => other.is_string(),
            &Val::Bool(_) => other.is_bool(),
            &Val::Stack(_) => other.is_stack(),
            &Val::Nil => other.is_nil(),
        }
    }

    pub fn type_string(&self) -> &'static str {
        match self {
            &Val::Int(_) => "int",
            &Val::Ident(_) => "identifier",
            &Val::Char(_) => "char",
            &Val::String(_) => "string",
            &Val::Bool(_) => "bool",
            &Val::Stack(_) => "local stack",
            &Val::Nil => "nil",
        }
    }

    pub fn compare(&self, other: &Val) -> Result<Ordering> {
        if !self.matches(other) {
            return Err(
                format!(
                    "attempted to compare a {} value against a {} value",
                    self.type_string(),
                    other.type_string()
                ).into(),
            );
        }

        match self {
            &Val::Int(i) => Ok(other.as_int().cmp(&i)),
            &Val::Ident(_) | &Val::String(_) | &Val::Bool(_) | &Val::Stack(_) | &Val::Nil => Err(
                format!(
                    "{} types may not be compared with ordinal operators",
                    self.type_string()
                ).into(),
            ),
            &Val::Char(c) => Ok(other.as_char().cmp(&c)),
        }
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Val::Int(i) => write!(f, "{}", i),
            &Val::Ident(ref s) => write!(f, "{}", s),
            &Val::Char(c) => write!(f, "{}", c),
            &Val::String(ref s) => write!(f, "{}", s),
            &Val::Bool(b) => write!(f, "{}", b),
            &Val::Stack(ref v) => {
                write!(
                    f,
                    "[{}]",
                    v.iter().map(Val::to_string).collect::<Vec<_>>().join(",")
                )
            }
            &Val::Nil => write!(f, "nil"),
        }
    }
}

impl From<IRVal> for Val {
    fn from(other: IRVal) -> Self {
        match other {
            IRVal::Int(i) => Val::Int(i),
            IRVal::Ident(i) => Val::Ident(i),
            IRVal::Char(c) => Val::Char(c),
            IRVal::String(s) => Val::String(s),
            IRVal::Bool(b) => Val::Bool(b),
            IRVal::Stack(s) => Val::Stack(s.into_iter().map(IRVal::into).collect()),
            IRVal::Nil => Val::Nil,
            IRVal::BakeBlock(_) => panic!("IRVal::BakeBlock variants may not be converted to Vals"),
        }
    }
}
