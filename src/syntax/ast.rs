use common::*;
use errors::*;
use syntax::token::*;
use std::rc::Rc;
use std::fmt::{Formatter, Debug, self};

pub type Tokens = Vec<Rc<Token>>;

pub struct AST;

pub trait ASTNode {
    fn tokens(&self) -> &[Rc<Token>];
    fn range(&self) -> Range {
        let tokens = self.tokens();
        //assert!(!tokens.is_empty());
        let start = tokens.first()
            .unwrap()
            .range()
            .start;
        let end = tokens.last()
            .unwrap()
            .range()
            .end;
        Range { start, end }
    }
}

/// A trait for the AST that defines the lookaheads of each node.
pub trait Lookaheads {
    fn lookaheads() -> &'static [TokenType];
}

macro_rules! lookaheads {
    ($($tt:expr),+) => {{
        lazy_static! {
            static ref TOKENS: Vec<TokenType> = vec![$($tt),+];
        };
        &TOKENS
    }};
}

#[derive(PartialEq, Clone, Debug)]
pub enum ItemType {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<Item>),
    Nil,
}

/// The Item AST node.
/// This is an atomic type; no further constructs are parsed above the "item"
/// level with this node.
///
/// An item may be an int, identifier, character, string, boolean, stack
/// literal, or nil.
#[derive(Clone, Debug)]
pub struct Item {
    tokens: Tokens,
    item_type: ItemType,
}

impl Item {
    pub fn new(tokens: Tokens, item_type: ItemType) -> Self {
        Item { tokens, item_type, }
    }

    pub fn item_type(&self) -> &ItemType {
        &self.item_type
    }
}

#[cfg(not(test))]
impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.item_type == other.item_type && self.tokens == other.tokens
    }
}

#[cfg(test)]
impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.item_type == other.item_type
    }
}

impl Lookaheads for Item {
    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::Int, TokenType::Ident, TokenType::Char,
                    TokenType::String, TokenType::KwT, TokenType::KwF,
                    TokenType::KwNil, TokenType::LBrack)
    }
}

impl ASTNode for Item {
    fn tokens(&self) -> &[Rc<Token>] {
        self.tokens
            .as_slice()
    }
}

impl From<Token> for Item {
    fn from(other: Token) -> Item {
        let other_str = other.as_str()
            .to_string();
        match other.token_type() {
            TokenType::Int => Item::new(vec![other.into_rc()], ItemType::Int(other_str.parse().unwrap())),
            TokenType::Ident => Item::new(vec![other.into_rc()], ItemType::Ident(other_str.to_string())),
            TokenType::Char => {
                let char_str = other.unescape();
                assert_eq!(char_str.len(), 1);
                Item::new(vec![other.into_rc()], ItemType::Char(char_str.chars().nth(0).unwrap()))
            },
            TokenType::String =>{
                let escaped = other.unescape();
                Item::new(vec![other.into_rc()], ItemType::String(escaped))
            },
            TokenType::KwT => Item::new(vec![other.into_rc()], ItemType::Bool(true)),
            TokenType::KwF => Item::new(vec![other.into_rc()], ItemType::Bool(false)),
            TokenType::KwNil => Item::new(vec![other.into_rc()], ItemType::Nil),
            _ => panic!("Token of type `{:?}` is incompatible to turn into an Item", other.token_type()),
        }
    }
}

