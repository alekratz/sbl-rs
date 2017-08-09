use errors::*;

use syntax::*;
use std::cmp::Ordering;
use std::fmt::{self, Formatter, Display};

#[derive(EnumGetters, EnumIsA, PartialEq, Clone, Debug)]
pub enum Val {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<Val>),
    BakeBlock(Block),
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
            &Val::BakeBlock(_) => other.is_bake_block(),
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
            &Val::BakeBlock(_) => "compile-time bake block",
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
            &Val::BakeBlock(_) => panic!("bake blocks should not be available for comparison"),
            &Val::Int(i) => Ok(other.int().cmp(&i)),
            &Val::Ident(_) | &Val::String(_) | &Val::Bool(_) | &Val::Stack(_) | &Val::Nil => Err(
                format!(
                    "{} types may not be compared with ordinal operators",
                    self.type_string()
                ).into(),
            ),
            &Val::Char(c) => Ok(other.char().cmp(&c)),
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
            &Val::BakeBlock(ref block) => write!(f, "bake {{ {:#?} }}", block),
            &Val::Nil => write!(f, "nil"),
        }
    }
}

impl From<Item> for Val {
    fn from(other: Item) -> Self {
        match other.into() {
            ItemType::Int(i) => Val::Int(i),
            ItemType::Ident(i) => Val::Ident(i),
            ItemType::Char(c) => Val::Char(c),
            ItemType::String(s) => Val::String(s),
            ItemType::Bool(b) => Val::Bool(b),
            ItemType::Stack(s) => Val::Stack(s.into_iter().map(Item::into).collect()),
            ItemType::Nil => Val::Nil,
        }
    }
}

impl<'a> From<&'a Item> for Val {
    fn from(other: &'a Item) -> Self {
        other.clone().into()
    }
}
