use ir::*;
use vm::*;
use syntax::*;

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

impl From<IR> for Bc {
    fn from(other: IR) -> Self {
        let new_type = match other.ir_type {
            IRType::Push => BcType::Push,
            IRType::PushL => BcType::PushL ,
            IRType::Pop => BcType::Pop ,
            IRType::PopN => BcType::PopN ,
            IRType::Load => BcType::Load ,
            IRType::JmpZ => BcType::JmpZ ,
            IRType::Jmp => BcType::Jmp ,
            IRType::Call => BcType::Call ,
            IRType::Ret => BcType::Ret ,
            IRType::Bake => panic!("IRType::Bake instructions cannot be converted to any BcType instruction"),
        };
        Bc {
            bc_type: new_type,
            val: other.val.map(Val::from),
            tokens: other.tokens,
        }
    }
}

