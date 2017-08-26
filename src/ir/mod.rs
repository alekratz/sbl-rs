pub mod fun;
pub mod val;

pub use ir::fun::*;
pub use ir::val::*;

use syntax::*;
use internal::Instruction;

impl Instruction for IR {}

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


