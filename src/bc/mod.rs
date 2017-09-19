pub mod fun;
pub mod val;

pub use self::fun::*;
pub use self::val::*;

use prelude::*;
use std::fmt::{self, Formatter, Display};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BCType {
    Push,           // push
    PushL,          // push list
    Pop,            // pop
    PopN,           // pop N items
    PopDiscard,     // pop, discarding the value
    Load,           // load variable
    StoreConst,     // store a constant value in a variable
    Jmp,            // jump unconditionally
    JmpZ,           // jump zero
    SymJmp,         // symbolic jump
    SymJmpZ,        // symbolic jump zero
    Call,           // call
    Ret,            // return
    Label,          // label (for symbolic jumps)
    Nop,            // no-op
}

impl Display for BCType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &BCType::Push => "PUSH",
                &BCType::PushL => "PUSHL",
                &BCType::Pop => "POP",
                &BCType::PopN => "POPN",
                &BCType::PopDiscard => "POP_DISCARD",
                &BCType::Load => "LOAD",
                &BCType::StoreConst => "STORE_CONST",
                &BCType::Jmp => "JMP",
                &BCType::JmpZ => "JMPZ",
                &BCType::SymJmp => "SYM_JMP",
                &BCType::SymJmpZ => "SYM_JMPZ",
                &BCType::Call => "CALL",
                &BCType::Ret => "RET",
                &BCType::Label => "LABEL",
                &BCType::Nop => "NOP",
            }
        )
    }
}

impl Instruction for BC { }

#[derive(Clone, PartialEq, Debug)]
pub struct BC {
    pub bc_type: BCType,
    pub tokens: Tokens,
    pub val: Option<BCVal>,
}

impl BC {
    pub fn push(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::PushAll(_));
        BC {
            bc_type: BCType::Push,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmp(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Address(_));
        BC {
            bc_type: BCType::Jmp,
            tokens,
            val: Some(val),
        }
    }

    pub fn jmpz(tokens: Tokens, val: BCVal) -> BC {
        assert_matches!(val, BCVal::Address(_));
        BC {
            bc_type: BCType::JmpZ,
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
            IRType::Push => return BC {
                bc_type: BCType::Push,
                val: Some(BCVal::PushAll(vec![other.val.map(BCVal::from)
                                       .expect("BCType::Push expects a value")])),
                tokens: other.tokens,
            },
            IRType::PushL => BCType::PushL,
            IRType::Pop => match other.val {
                Some(IRVal::Ident(_)) => BCType::Pop,
                Some(IRVal::Int(_)) => BCType::PopN,
                Some(IRVal::Nil) => BCType::PopDiscard,
                _ => unreachable!(),
            },
            IRType::Load => BCType::Load,
            IRType::Jmp => BCType::SymJmp,
            IRType::JmpZ => BCType::SymJmpZ,
            IRType::Call => BCType::Call,
            IRType::Ret => BCType::Ret,
            IRType::Bake => panic!("IRType::Bake instructions cannot be converted to any BCType instruction"),
            IRType::Label => BCType::Label,
            IRType::Nop => BCType::Nop,
        };
        BC {
            bc_type: new_type,
            val: other.val.map(BCVal::from),
            tokens: other.tokens,
        }
    }
}

