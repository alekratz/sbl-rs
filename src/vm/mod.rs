mod compile;
mod vm;
mod builtins;

pub use self::compile::*;
pub use self::vm::*;

use syntax::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(EnumMethods, PartialEq, Clone, Debug)]
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
    /*
    pub fn int(&self) -> i64 { if let &Val::Int(ref v) = self { *v } else { panic!("called int() on {:?}", self) } }
    pub fn ident(&self) -> &str { if let &Val::Ident(ref v) = self { v } else { panic!("called ident() on {:?}", self) } }
    pub fn char(&self) -> char { if let &Val::Char(ref v) = self { *v } else { panic!("called char() on {:?}", self) } }
    pub fn string(&self) -> &str { if let &Val::String(ref v) = self { v } else { panic!("called string() on {:?}", self) } }
    pub fn bool(&self) -> bool { if let &Val::Bool(ref v) = self { *v } else { panic!("called bool() on {:?}", self) } }
    pub fn stack(&self) -> &[Val] { if let &Val::Stack(ref v) = self { v } else { panic!("called stack() on {:?}", self) } }
    pub fn is_nil(&self) -> bool { self == &Val::Nil }
    */
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
