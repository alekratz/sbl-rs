mod compile;
mod vm;
mod builtins;
mod foreign;

pub use self::compile::*;
pub use self::vm::*;
pub(in vm) use self::builtins::*;

use errors::*;
use syntax::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{Formatter, Display, self};
use std::cmp::Ordering;

#[derive(EnumGetters, EnumIsA, PartialEq, Clone, Debug)]
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
            return Err("attempted to compare a {} value against a {} value".into());
        }

        match self {
            &Val::Int(i) => Ok(other.int().cmp(&i)),
            &Val::Ident(_) | &Val::String(_) | &Val::Bool(_) | &Val::Stack(_) | &Val::Nil =>
                Err(format!("{} types may not be compared with ordinal operators", self.type_string()).into()),
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
            &Val::Stack(ref v) => write!(f, "[{}]", v.iter().map(Val::to_string).collect::<Vec<_>>().join(",")),
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
            ItemType::Stack(s) =>
                Val::Stack(s.into_iter().map(Item::into).collect()),
            ItemType::Nil => Val::Nil,
        }
    }
}

impl<'a> From<&'a Item> for Val {
    fn from(other: &'a Item) -> Self {
        other.clone().into()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BcType {
    Push,
    PushL,
    Pop,
    PopN,
    Load,
    JmpZ,
    Jmp,
    Call,
    Ret,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bc {
    bc_type: BcType,
    tokens: Tokens,
    val: Option<Val>,
}

impl Bc {
    pub fn bc_type(&self) -> &BcType {
        &self.bc_type
    }

    pub fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    pub fn val(&self) -> Option<&Val> {
        self.val.as_ref()
    }

    pub fn val_clone(&self) -> Option<Val> {
        self.val.clone()
    }

    pub fn push(tokens: Tokens, val: Val) -> Bc {
        Bc { bc_type: BcType::Push, tokens, val: Some(val) }
    }

    pub fn pushl(tokens: Tokens) -> Bc {
        Bc { bc_type: BcType::PushL, tokens, val: None }
    }

    pub fn pop(tokens: Tokens, val: Val) -> Bc {
        Bc { bc_type: BcType::Pop, tokens, val: Some(val) }
    }

    pub fn popn(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc { bc_type: BcType::PopN, tokens, val: Some(val) }
    }

    pub fn load(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Ident(_));
        Bc { bc_type: BcType::Load, tokens, val: Some(val) }
    }

    pub fn jmpz(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc { bc_type: BcType::JmpZ, tokens, val: Some(val) }
    }
    
    pub fn jmp(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc { bc_type: BcType::Jmp, tokens, val: Some(val) }
    }

    pub fn call(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Ident(_));
        Bc { bc_type: BcType::Call, tokens, val: Some(val) }
    }

    pub fn ret(tokens: Tokens) -> Bc {
        Bc { bc_type: BcType::Ret, tokens, val: None }
    }
}

pub type BcBody = Vec<Bc>;

#[derive(Clone, Debug)]
pub struct Fun {
    name: String,
    body: BcBody,
    tokens: Tokens,
}

impl Fun {
    pub fn new(name: String, body: BcBody, tokens: Tokens) -> Self {
        Fun { name, body, tokens }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    pub fn body(&self) -> &[Bc] {
        &self.body
    }
}

pub type FunTable = HashMap<String, Rc<Fun>>;
