use common::*;
use errors::*;
use syntax::token::*;
use std::rc::Rc;
use std::fmt::{Formatter, Debug, self};

pub type Tokens = Vec<Rc<Token>>;

pub struct AST;

pub trait ASTNode {
    fn lookaheads() -> &'static [TokenType];
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

macro_rules! lookaheads {
    (@ TokenType::$head:ident $($tail:tt)*) => {{
        let mut tail = lookaheads!(@ $($tail)*);
        tail.push(TokenType::$head);
        tail
    }};
    (@ $head:ident $($tail:tt)*) => {{
        let mut tail = lookaheads!(@ $($tail)*);
        tail.extend_from_slice($head::lookaheads().clone());
        tail
    }};
    (@) => { vec![] };

    ($($tt:tt)+) => {{
        lazy_static! {
            static ref TOKENS: Vec<TokenType> = lookaheads!(@ $($tt)+);
        };
        &TOKENS
    }};
}

//
// Items
//

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

impl ASTNode for Item {
    fn tokens(&self) -> &[Rc<Token>] {
        self.tokens
            .as_slice()
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::Int TokenType::Ident TokenType::Char
                    TokenType::String TokenType::KwT TokenType::KwF
                    TokenType::KwNil TokenType::LBrack)
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

//
// Stack actions
//

pub enum StackAction {
    Push(Item),
    Pop(Tokens, Item),
}

impl StackAction {
    pub fn item(&self) -> &Item {
        match self {
            &StackAction::Push(ref i) => i,
            &StackAction::Pop(_, ref i) => i,
        }
    }
}

impl ASTNode for StackAction {
    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(Item TokenType::Dot)
    }

    fn tokens(&self) -> &[Rc<Token>] {
        match self {
            &StackAction::Push(ref i) => i.tokens(),
            &StackAction::Pop(ref t, _) => t,
        }
    }
}

//
// Stack statements
//

pub struct StackStmt {
    tokens: Tokens,
    stack_actions: Vec<StackAction>,
}

impl StackStmt  {
    pub fn new(tokens: Tokens, stack_actions: Vec<StackAction>) -> Self {
        StackStmt {
            tokens,
            stack_actions,
        }
    }

    pub fn stack_actions(&self) -> &[StackAction] {
        &self.stack_actions
    }
}

impl ASTNode for StackStmt {
    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(StackStmt TokenType::Semi)
    }

    fn tokens(&self) -> &[Rc<Token>] {
        &self.tokens
    }
}

