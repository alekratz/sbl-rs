mod compile;

pub use self::compile::*;

use syntax::*;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub enum Val {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<Val>),
    Nil,
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

pub struct Bc {
    bc_type: BcType,
    tokens: Tokens,
    val: Option<Val>,
}

impl Bc {
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

pub type FunTable = HashMap<String, Fun>;
