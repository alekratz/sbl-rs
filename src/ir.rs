use vm::*;
use syntax::*;
use errors::*;
use std::cmp::Ordering;
use std::fmt::{self, Formatter, Display};
use std::collections::HashMap;

#[derive(EnumAsGetters, EnumIsA, PartialEq, Clone, Debug)]
pub enum IRVal {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<IRVal>),
    Nil,
    BakeBlock(Block),
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
            &IRVal::Nil => write!(f, "nil"),
            &IRVal::BakeBlock(ref b) => write!(f, "bake block {{ {:#?} }}", b),
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IRType {
    Push,
    PushL,
    Pop,
    PopN,
    Load,
    JmpZ,
    Jmp,
    Call,
    Ret,
    Bake,
}

impl Display for IRType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &IRType::Push => "PUSH",
                &IRType::PushL => "PUSHL",
                &IRType::Pop => "POP",
                &IRType::PopN => "POPN",
                &IRType::Load => "LOAD",
                &IRType::JmpZ => "JMPZ",
                &IRType::Jmp => "JMP",
                &IRType::Call => "CALL",
                &IRType::Ret => "RET",
                &IRType::Bake => "BAKE",
            }
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct IR {
    pub ir_type: IRType,
    pub tokens: Tokens,
    pub val: Option<IRVal>,
}

impl IR {
    pub fn push(tokens: Tokens, val: IRVal) -> IR {
        IR {
            ir_type: IRType::Push,
            tokens,
            val: Some(val),
        }
    }

    pub fn pushl(tokens: Tokens) -> IR {
        IR {
            ir_type: IRType::PushL,
            tokens,
            val: None,
        }
    }

    pub fn pop(tokens: Tokens, val: IRVal) -> IR {
        IR {
            ir_type: IRType::Pop,
            tokens,
            val: Some(val),
        }
    }

    pub fn popn(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::Int(_));
        IR {
            ir_type: IRType::PopN,
            tokens,
            val: Some(val),
        }
    }

    pub fn load(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::Ident(_));
        IR {
            ir_type: IRType::Load,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmpz(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::Int(_));
        IR {
            ir_type: IRType::JmpZ,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmp(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::Int(_));
        IR {
            ir_type: IRType::Jmp,
            tokens,
            val: Some(val),
        }
    }

    pub fn call(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::Ident(_));
        IR {
            ir_type: IRType::Call,
            tokens,
            val: Some(val),
        }
    }

    pub fn ret(tokens: Tokens) -> IR {
        IR {
            ir_type: IRType::Ret,
            tokens,
            val: None,
        }
    }

    pub fn bake(tokens: Tokens, val: IRVal) -> IR {
        assert_matches!(val, IRVal::BakeBlock(_));
        IR {
            ir_type: IRType::Bake,
            tokens,
            val: Some(val),
        }
    }
}

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

#[derive(EnumIsA)]
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
            panic!("Fun::as_user_fun() called on non-UserFun item")
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

pub type IRFunTable = HashMap<String, IRFun>;
