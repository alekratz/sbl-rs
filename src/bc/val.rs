use prelude::*;
use std::cmp::Ordering;
use std::fmt::{self, Formatter, Display};

#[derive(EnumAsGetters, EnumIsA, PartialEq, Clone, Debug)]
pub enum BCVal {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<BCVal>),
    PushAll(Vec<BCVal>),
    Nil,
}

impl BCVal {
    pub fn matches(&self, other: &Self) -> bool {
        match self {
            &BCVal::Int(_) => other.is_int(),
            &BCVal::Ident(_) => other.is_ident(),
            &BCVal::Char(_) => other.is_char(),
            &BCVal::String(_) => other.is_string(),
            &BCVal::Bool(_) => other.is_bool(),
            &BCVal::Stack(_) => other.is_stack(),
            &BCVal::PushAll(_) => other.is_push_all(),
            &BCVal::Nil => other.is_nil(),
        }
    }

    pub fn type_string(&self) -> &'static str {
        match self {
            &BCVal::Int(_) => "int",
            &BCVal::Ident(_) => "identifier",
            &BCVal::Char(_) => "char",
            &BCVal::String(_) => "string",
            &BCVal::Bool(_) => "bool",
            &BCVal::Stack(_) => "local stack",
            &BCVal::PushAll(_) => "push collection",
            &BCVal::Nil => "nil",
        }
    }

    pub fn compare(&self, other: &BCVal) -> Result<Ordering> {
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
            &BCVal::Int(i) => Ok(other.as_int().cmp(&i)), 
            &BCVal::Char(c) => Ok(other.as_char().cmp(&c)),
            _ => Err(
                format!(
                    "{} types may not be compared with ordinal operators",
                    self.type_string()
                ).into(),
            ),
        }
    }

    /// Appends the contents of one stack value to another.
    pub fn append(&mut self, other: &mut BCVal) {
        assert!(self.is_push_all() && other.is_push_all());
        if let &mut BCVal::PushAll(ref mut mine) = self {
            if let &mut BCVal::PushAll(ref mut theirs) = other {
                mine.append(theirs);
            }
            else { unreachable!() }
        }
        else { unreachable!() }
    }
}

impl Display for BCVal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &BCVal::Int(i) => write!(f, "{}", i),
            &BCVal::Ident(ref s) => write!(f, "{}", s),
            &BCVal::Char(c) => write!(f, "{}", c),
            &BCVal::String(ref s) => write!(f, "{}", s),
            &BCVal::Bool(b) => write!(f, "{}", b),
            &BCVal::Stack(ref v) | &BCVal::PushAll(ref v) => {
                write!(
                    f,
                    "[{}]",
                    v.iter().map(BCVal::to_string).collect::<Vec<_>>().join(",")
                )
            }
            &BCVal::Nil => write!(f, "nil"),
        }
    }
}

impl From<IRVal> for BCVal {
    fn from(other: IRVal) -> Self {
        match other {
            IRVal::Int(i) => BCVal::Int(i),
            IRVal::Ident(i) => BCVal::Ident(i),
            IRVal::Char(c) => BCVal::Char(c),
            IRVal::String(s) => BCVal::String(s),
            IRVal::Bool(b) => BCVal::Bool(b),
            IRVal::Stack(s) => BCVal::Stack(s.into_iter().map(IRVal::into).collect()),
            IRVal::Nil => BCVal::Nil,
            IRVal::BakeBlock(_) => panic!("IRVal::BakeBlock variants may not be converted to BCVals"),
        }
    }
}
