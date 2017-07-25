mod vm;
mod builtins;
mod foreign;

pub use self::vm::*;
pub use self::builtins::*;

use errors::*;
use syntax::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{self, Formatter, Display};
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

impl Display for BcType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &BcType::Push => "PUSH",
                &BcType::PushL => "PUSHL",
                &BcType::Pop => "POP",
                &BcType::PopN => "POPN",
                &BcType::Load => "LOAD",
                &BcType::JmpZ => "JMPZ",
                &BcType::Jmp => "JMP",
                &BcType::Call => "CALL",
                &BcType::Ret => "RET",
            }
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bc {
    pub bc_type: BcType,
    pub tokens: Tokens,
    pub val: Option<Val>,
}

impl Bc {
    pub fn push(tokens: Tokens, val: Val) -> Bc {
        Bc {
            bc_type: BcType::Push,
            tokens,
            val: Some(val),
        }
    }

    pub fn pushl(tokens: Tokens) -> Bc {
        Bc {
            bc_type: BcType::PushL,
            tokens,
            val: None,
        }
    }

    pub fn pop(tokens: Tokens, val: Val) -> Bc {
        Bc {
            bc_type: BcType::Pop,
            tokens,
            val: Some(val),
        }
    }

    pub fn popn(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc {
            bc_type: BcType::PopN,
            tokens,
            val: Some(val),
        }
    }

    pub fn load(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Ident(_));
        Bc {
            bc_type: BcType::Load,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmpz(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc {
            bc_type: BcType::JmpZ,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmp(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Int(_));
        Bc {
            bc_type: BcType::Jmp,
            tokens,
            val: Some(val),
        }
    }

    pub fn call(tokens: Tokens, val: Val) -> Bc {
        assert_matches!(val, Val::Ident(_));
        Bc {
            bc_type: BcType::Call,
            tokens,
            val: Some(val),
        }
    }

    pub fn ret(tokens: Tokens) -> Bc {
        Bc {
            bc_type: BcType::Ret,
            tokens,
            val: None,
        }
    }
}

pub type BcBody = Vec<Bc>;

#[derive(Clone, Debug)]
pub struct UserFun {
    pub name: String,
    pub body: BcBody,
    pub tokens: Tokens,
}

impl UserFun {
    pub fn new(name: String, body: BcBody, tokens: Tokens) -> Self {
        UserFun { name, body, tokens }
    }

    pub fn dump(&self) {
        for bc in &self.body {
            eprintln!("{:6} {}", &bc.bc_type.to_string(),
                if let Some(ref payload) = bc.val { format!("{:?}", payload) } else { format!("") });
        }
    }

    pub fn replace_body(&mut self, body: BcBody) {
        self.body = body;
    }
}

#[derive(EnumIsA)]
pub enum Fun {
    UserFun(Rc<UserFun>),
    ForeignFun(ForeignFn),
    BuiltinFun(&'static BuiltinFun),
}

impl Fun {
    pub fn user_fun(&self) -> &UserFun {
        if let &Fun::UserFun(ref fun) = self {
            fun
        }
        else {
            panic!("Fun::user_fun() called on non-UserFun item")
        }
    }
}

pub type FunTable = HashMap<String, Fun>;
pub type FunRcTable = HashMap<String, Rc<Fun>>;
