use prelude::*;
use std::cmp::Ordering;
use std::fmt::{self, Formatter, Display};

#[derive(EnumIntoGetters, EnumAsGetters, EnumIsA, PartialEq, Clone, Debug)]
pub enum IRVal {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<IRVal>),
    Address(usize),
    Nil,
    BakeBlock(IRBody),
}

impl IRVal {
    pub fn matches(&self, other: &Self) -> bool {
        match self {
            &IRVal::Int(_) => other.is_int(),
            &IRVal::Ident(_) => other.is_ident(),
            &IRVal::Char(_) => other.is_char(),
            &IRVal::String(_) => other.is_string(),
            &IRVal::Bool(_) => other.is_bool(),
            &IRVal::Stack(_) => other.is_stack(),
            &IRVal::Address(_) => other.is_address(),
            &IRVal::Nil => other.is_nil(),
            &IRVal::BakeBlock(_) => other.is_bake_block(),
        }
    }

    pub fn type_string(&self) -> &'static str {
        match self {
            &IRVal::Int(_) => "int",
            &IRVal::Ident(_) => "identifier",
            &IRVal::Char(_) => "char",
            &IRVal::String(_) => "string",
            &IRVal::Bool(_) => "bool",
            &IRVal::Stack(_) => "local stack",
            &IRVal::Address(_) => "address",
            &IRVal::Nil => "nil",
            &IRVal::BakeBlock(_) => "bake block",
        }
    }

    pub fn compare(&self, other: &IRVal) -> Result<Ordering> {
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
            &IRVal::Int(i) => Ok(other.as_int().cmp(&i)),
            &IRVal::Address(a) => Ok(other.as_address().cmp(&a)),
            &IRVal::Ident(_) | &IRVal::String(_) | &IRVal::Bool(_) | &IRVal::Stack(_) | &IRVal::Nil => Err(
                format!(
                    "{} types may not be compared with ordinal operators",
                    self.type_string()
                ).into(),
            ),
            &IRVal::Char(c) => Ok(other.as_char().cmp(&c)),
            &IRVal::BakeBlock(_) => {
                panic!("bake blocks may not be formatted");
            },
        }
    }
}

impl Display for IRVal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &IRVal::Int(i) => write!(f, "{}", i),
            &IRVal::Ident(ref s) => write!(f, "{}", s),
            &IRVal::Char(c) => write!(f, "{}", c),
            &IRVal::String(ref s) => write!(f, "{}", s),
            &IRVal::Bool(b) => write!(f, "{}", b),
            &IRVal::Stack(ref v) => {
                write!(
                    f,
                    "[{}]",
                    v.iter().map(IRVal::to_string).collect::<Vec<_>>().join(",")
                )
            }
            &IRVal::Address(a) => write!(f, "0x{:X}", a),
            &IRVal::Nil => write!(f, "nil"),
            &IRVal::BakeBlock(ref b) => write!(f, "bake block {{ {:#?} }}", b),
        }
    }
}

impl From<BCVal> for IRVal {
    fn from(other: BCVal) -> Self {
        match other {
            BCVal::Int(i) => IRVal::Int(i),
            BCVal::Ident(i) => IRVal::Ident(i),
            BCVal::Char(c) => IRVal::Char(c),
            BCVal::String(s) => IRVal::String(s),
            BCVal::Bool(b) => IRVal::Bool(b),
            BCVal::Stack(s) => IRVal::Stack(s.into_iter().map(BCVal::into).collect()),
            BCVal::Address(a) => IRVal::Address(a),
            BCVal::PushAll(_) => panic!("BCVal::PushAll values cannot be converted to an IRVal"),
            BCVal::Nil => IRVal::Nil,
        }
    }
}

impl From<Item> for IRVal {
    fn from(other: Item) -> Self {
        match other.into() {
            ItemType::Int(i) => IRVal::Int(i),
            ItemType::Ident(i) => IRVal::Ident(i),
            ItemType::Char(c) => IRVal::Char(c),
            ItemType::String(s) => IRVal::String(s),
            ItemType::Bool(b) => IRVal::Bool(b),
            ItemType::Stack(s) => IRVal::Stack(s.into_iter().map(Item::into).collect()),
            ItemType::Nil => IRVal::Nil,
        }
    }
}

impl<'a> From<&'a Item> for IRVal {
    fn from(other: &'a Item) -> Self {
        other.clone().into()
    }
}

