use ir::*;
use vm::*;
use syntax::*;
use std::fmt::{self, Formatter, Display};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BCType {
    Push,
    PushL,
    PushA,
    Pop,
    PopN,
    Load,
    JmpZ,
    Jmp,
    Call,
    Ret,
}

impl Display for BCType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &BCType::Push => "PUSH",
                &BCType::PushA => "PUSHA",
                &BCType::PushL => "PUSHL",
                &BCType::Pop => "POP",
                &BCType::PopN => "POPN",
                &BCType::Load => "LOAD",
                &BCType::JmpZ => "JMPZ",
                &BCType::Jmp => "JMP",
                &BCType::Call => "CALL",
                &BCType::Ret => "RET",
            }
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct BC {
    pub bc_type: BCType,
    pub tokens: Tokens,
    pub val: Option<BCVal>,
}

impl BC {
    pub fn push(tokens: Tokens, val: BCVal) -> BC {
        BC {
            bc_type: BCType::Push,
            tokens,
            val: Some(val),
        }
    }

    pub fn pusha(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Stack(_));
        BC {
            bc_type: BCType::Push,
            tokens,
            val: Some(val),
        }
    }

    pub fn pushl(tokens: Tokens) -> BC {
        BC {
            bc_type: BCType::PushL,
            tokens,
            val: None,
        }
    }

    pub fn pop(tokens: Tokens, val: BCVal) -> BC {
        BC {
            bc_type: BCType::Pop,
            tokens,
            val: Some(val),
        }
    }

    pub fn popn(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Int(_));
        BC {
            bc_type: BCType::PopN,
            tokens,
            val: Some(val),
        }
    }

    pub fn load(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Ident(_));
        BC {
            bc_type: BCType::Load,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmpz(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Int(_));
        BC {
            bc_type: BCType::JmpZ,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmp(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Int(_));
        BC {
            bc_type: BCType::Jmp,
            tokens,
            val: Some(val),
        }
    }

    pub fn call(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Ident(_));
        BC {
            bc_type: BCType::Call,
            tokens,
            val: Some(val),
        }
    }

    pub fn ret(tokens: Tokens) -> BC {
        BC {
            bc_type: BCType::Ret,
            tokens,
            val: None,
        }
    }
}

pub type BCBody = Vec<BC>;

impl From<IR> for BC {
    fn from(other: IR) -> Self {
        let new_type = match other.ir_type {
            IRType::Push => BCType::Push,
            IRType::PushL => BCType::PushL,
            IRType::Pop => BCType::Pop,
            IRType::PopN => BCType::PopN,
            IRType::Load => BCType::Load,
            IRType::JmpZ => BCType::JmpZ,
            IRType::Jmp => BCType::Jmp,
            IRType::Call => BCType::Call,
            IRType::Ret => BCType::Ret,
            IRType::Bake => panic!("IRType::Bake instructions cannot be converted to any BCType instruction"),
        };
        BC {
            bc_type: new_type,
            val: other.val.map(BCVal::from),
            tokens: other.tokens,
        }
    }
}

