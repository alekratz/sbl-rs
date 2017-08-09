use common::*;
use syntax::token::*;
#[cfg(test)]
use std::fmt::{self, Formatter, Debug};

pub type Tokens = Vec<RcToken>;

/// A trait which provides special functions for `Tokens`, aka `Vec<RcToken>`.
pub trait TokensVec {
    fn append_node<T: ASTNode>(&mut self, node: &T);
    fn range(&self) -> Range;
    fn contains_bake_token(&self) -> bool;
}

impl TokensVec for Tokens {
    fn append_node<T: ASTNode>(&mut self, node: &T) {
        self.extend_from_slice(node.tokens());
    }

    fn range(&self) -> Range {
        assert!(!self.is_empty());
        let first = self.first().unwrap().range();
        let last = self.last().unwrap().range();
        Range::new(first.start, last.end)
    }

    fn contains_bake_token(&self) -> bool {
        for t in self {
            if t.token_type() == TokenType::KwBake {
                return true;
            }
        }
        false
    }
}

pub trait ASTNode {
    fn lookaheads() -> &'static [TokenType];
    fn tokens(&self) -> &[RcToken];
    fn range(&self) -> Range {
        let tokens = self.tokens();
        let start = tokens.first().unwrap().range().start;
        let end = tokens.last().unwrap().range().end;
        Range { start, end }
    }
}

macro_rules! lookaheads {
    (@ TokenType::$head:ident ( $($expr:expr),+ ) $($tail:tt)*) => {{
        let mut tail = lookaheads!(@ $($tail)*);
        tail.push(TokenType::$head ( $($expr),+ ));
        tail
    }};
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

#[derive(EnumIsA, PartialEq, Clone, Debug)]
pub enum ItemType {
    Int(i64),
    Ident(String),
    Char(char),
    String(String),
    Bool(bool),
    Stack(Vec<Item>),
    Nil,
}

impl ItemType {
    pub fn type_string(&self) -> &'static str {
        match self {
            &ItemType::Int(_) => "int",
            &ItemType::Ident(_) => "identifier",
            &ItemType::Char(_) => "char",
            &ItemType::String(_) => "string",
            &ItemType::Bool(_) => "bool",
            &ItemType::Stack(_) => "local stack",
            &ItemType::Nil => "nil",
        }
    }
}

impl From<Item> for ItemType {
    fn from(item: Item) -> Self {
        item.item_type
    }
}

/// The Item AST node.
/// This is an atomic type; no further constructs are parsed above the "item"
/// level with this node.
///
/// An item may be an int, identifier, character, string, boolean, stack
/// literal, or nil.
#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct Item {
    pub tokens: Tokens,
    pub item_type: ItemType,
}

impl Item {
    pub fn new(tokens: Tokens, item_type: ItemType) -> Self {
        Item { tokens, item_type }
    }

    pub fn is_const(&self) -> bool {
        match self.item_type {
            ItemType::Ident(_) => false,
            ItemType::Stack(ref s) => s.iter().all(Item::is_const),
            _ => true,
        }
    }
}

#[cfg(test)]
impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.item_type == other.item_type
    }
}

#[cfg(test)]
impl Debug for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Item {{ {:?} }}", self.item_type)
    }
}

impl ASTNode for Item {
    fn tokens(&self) -> &[RcToken] {
        self.tokens.as_slice()
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::Int TokenType::Ident TokenType::Char
                    TokenType::String TokenType::KwT TokenType::KwF
                    TokenType::KwNil TokenType::LBrack
                    TokenType::BasedInt(2)
                    TokenType::BasedInt(8)
                    TokenType::BasedInt(16))
    }
}

impl From<Token> for Item {
    fn from(other: Token) -> Item {
        let other_str = other.as_str().to_string();
        match other.token_type() {
            TokenType::Int => {
                Item::new(
                    vec![other.into_rc()],
                    ItemType::Int(other_str.parse().unwrap()),
                )
            }
            TokenType::BasedInt(base) => {
                Item::new(
                    vec![other.into_rc()],
                    ItemType::Int(i64::from_str_radix(&other_str[2..], base as u32).unwrap()),
                )
            }
            TokenType::Ident => {
                Item::new(
                    vec![other.into_rc()],
                    ItemType::Ident(other_str.to_string()),
                )
            }
            TokenType::Char => {
                let char_str = other.unescape();
                assert_eq!(char_str.len(), 1);
                Item::new(
                    vec![other.into_rc()],
                    ItemType::Char(char_str.chars().nth(0).unwrap()),
                )
            }
            TokenType::String => {
                let escaped = other.unescape();
                Item::new(vec![other.into_rc()], ItemType::String(escaped))
            }
            TokenType::KwT => Item::new(vec![other.into_rc()], ItemType::Bool(true)),
            TokenType::KwF => Item::new(vec![other.into_rc()], ItemType::Bool(false)),
            TokenType::KwNil => Item::new(vec![other.into_rc()], ItemType::Nil),
            _ => {
                panic!(
                    "Token of type `{:?}` is incompatible to turn into an Item",
                    other.token_type()
                )
            }
        }
    }
}

//
// Stack actions
//

#[derive(Clone, EnumIsA)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
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

    fn tokens(&self) -> &[RcToken] {
        match self {
            &StackAction::Push(ref i) => i.tokens(),
            &StackAction::Pop(ref t, _) => t,
        }
    }
}

#[cfg(test)]
impl PartialEq for StackAction {
    fn eq(&self, other: &Self) -> bool {
        use self::StackAction::*;
        (self.item() == other.item()) &&
            match *self {
                Push(_) => other.is_push(),
                Pop(_, _) => other.is_pop(),
            }
    }
}

#[cfg(test)]
impl Debug for StackAction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &StackAction::Push(_) => write!(f, "Push {{ {:?} }}", self.item()),
            &StackAction::Pop(_, _) => write!(f, "Pop {{ {:?} }}", self.item()),
        }
    }
}

//
// Statements
//

#[derive(Clone, Debug)]
#[cfg_attr(not(test), derive(PartialEq))]
pub enum Stmt {
    Stack(StackStmt),
    Br(BrStmt),
    Loop(LoopStmt),
    Bake(BakeStmt),
}

impl ASTNode for Stmt {
    fn tokens(&self) -> &[RcToken] {
        match *self {
            Stmt::Stack(ref s) => s.tokens(),
            Stmt::Br(ref s) => s.tokens(),
            Stmt::Loop(ref s) => s.tokens(),
            Stmt::Bake(ref s) => s.tokens(),
        }
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(StackStmt BrStmt ElStmt LoopStmt)
    }
}

#[cfg(test)]
impl PartialEq for Stmt {
    fn eq(&self, other: &Self) -> bool {
        use self::Stmt::*;
        match self {
            &Stack(ref s) => {
                if let &Stack(ref o) = other {
                    s == o
                } else {
                    false
                }
            }
            &Br(ref s) => {
                if let &Br(ref o) = other {
                    s == o
                } else {
                    false
                }
            }
            &Loop(ref s) => {
                if let &Loop(ref o) = other {
                    s == o
                } else {
                    false
                }
            }
        }
    }
}

macro_rules! from_stmt {
    ($rule:ident, $name:ident) => {
        impl From<Stmt> for $name {
            fn from(stmt: Stmt) -> Self {
                match stmt {
                    Stmt::$rule(s) => s,
                    _ => panic!(format!(concat!("called ", stringify!($name), "::from() for mismatched Stmt ({:?})"),
                            stmt)),
                }
            }
        }

        /*
        impl From<$name> for Stmt {
            fn from(other: $name) -> Self {
                Stmt::$rule(other)
            }
        }
        */
    };
}

from_stmt!(Stack, StackStmt);
from_stmt!(Br, BrStmt);
from_stmt!(Loop, LoopStmt);
from_stmt!(Bake, BakeStmt);

//
// Stack statements
//

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct StackStmt {
    pub tokens: Tokens,
    pub stack_actions: Vec<StackAction>,
}

impl StackStmt {
    pub fn new(tokens: Tokens, stack_actions: Vec<StackAction>) -> Self {
        StackStmt {
            tokens,
            stack_actions,
        }
    }
}

impl ASTNode for StackStmt {
    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(StackAction TokenType::Semi)
    }

    fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }
}

#[cfg(test)]
impl PartialEq for StackStmt {
    fn eq(&self, other: &Self) -> bool {
        self.stack_actions == other.stack_actions
    }
}

#[cfg(test)]
impl Debug for StackStmt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "StackStmt {{ {:?} }}", self.stack_actions)
    }
}

//
// Block statements
//

macro_rules! block_stmt {
    (@ $name:ident new => ($($param:ident : $type:ty ),*) $($tail:tt)* ) => {
        impl $name {
            pub fn new(tokens: Tokens $( , $param: $type )*) -> Self {
                $name {
                    tokens,
                    $( $param , )*
                }
            }
        }

        #[cfg(test)]
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                true $( && self.$param == other.$param )*
            }
        }

        #[cfg(test)]
        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                $(
                    write!(f, "{} {{ {:?} }}", stringify!($param), self.$param)
                        .unwrap();
                )*
                Ok(())
            }
        }

        block_stmt!(@ $name $($tail)*);
    };
    (@ $name:ident lookaheads => ($($lookaheads:tt)+) $($tail:tt)*) => {
        impl ASTNode for $name {
            fn tokens(&self) -> &[RcToken] {
                &self.tokens
            }

            fn lookaheads() -> &'static [TokenType] {
                lookaheads!($($lookaheads)+)
            }
        }

        block_stmt!(@ $name $($tail)*);
    };
    (@ $name:ident) => {};

    ($name:ident $($tail:tt)+) => {
        block_stmt!(@ $name $($tail)+);
    };
}

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct Block {
    pub tokens: Tokens,
    pub block: Vec<Stmt>,
}

block_stmt!(Block
            new => (block: Vec<Stmt>)
            lookaheads => (TokenType::LBrace));

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct BrStmt {
    pub tokens: Tokens,
    pub block: Block,
    pub el_stmt: Option<ElStmt>,
}

block_stmt!(BrStmt
            new => (block: Block, el_stmt: Option<ElStmt>)
            lookaheads => (TokenType::KwBr));

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct ElStmt {
    pub tokens: Tokens,
    pub block: Block,
}

block_stmt!(ElStmt
            new => (block: Block)
            lookaheads => (TokenType::KwEl));

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct LoopStmt {
    pub tokens: Tokens,
    pub block: Block,
}

block_stmt!(LoopStmt
            new => (block: Block)
            lookaheads => (TokenType::KwLoop));

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct BakeStmt {
    pub tokens: Tokens,
    pub block: Block,
}

block_stmt!(BakeStmt
            new => (block: Block)
            lookaheads => (TokenType::KwBake));

//
// Top level statements
//

#[derive(EnumGetters, Clone, PartialEq, Debug)]
pub enum TopLevel {
    FunDef(FunDef),
    Import(Import),
    Foreign(Foreign),
}

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct FunDef {
    pub tokens: Tokens,
    pub name: String,
    pub block: Block,
}

impl FunDef {
    pub fn new(tokens: Tokens, name: String, block: Block) -> Self {
        FunDef {
            tokens,
            name,
            block,
        }
    }
}

impl ASTNode for FunDef {
    fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::Ident)
    }
}

#[cfg(test)]
impl PartialEq for FunDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.block == other.block
    }
}

#[cfg(test)]
impl Debug for FunDef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FunDef {{ name: {:?} block: {:?} }}",
            self.name,
            self.block
        )
    }
}

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct Import {
    pub tokens: Tokens,
    pub path: String,
}

impl Import {
    pub fn new(tokens: Tokens, path: String) -> Self {
        Import { tokens, path }
    }
}

impl ASTNode for Import {
    fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::KwImport)
    }
}

#[cfg(test)]
impl PartialEq for Import {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

#[cfg(test)]
impl Debug for Import {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Import {{ path: {:?} }}", self.path)
    }
}

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct ForeignFn {
    pub tokens: Tokens,
    /// Name of the foreign function to call.
    pub name: String,
    /// Name of the library where the foreign function exists.
    pub lib: String,
    /// List of the parameters that this call takes.
    pub params: Vec<ItemType>,
    /// The return type of the function.
    pub return_type: ItemType,
}

impl ForeignFn {
    pub fn new(
        tokens: Tokens,
        name: String,
        lib: String,
        params: Vec<ItemType>,
        return_type: ItemType,
    ) -> Self {
        ForeignFn {
            tokens,
            name,
            lib,
            params,
            return_type,
        }
    }
}

impl ASTNode for ForeignFn {
    fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::Ident)
    }
}

#[cfg(test)]
impl Debug for ForeignFn {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "ForeignFn {{ name: {} lib: {} params: {:?} return_type: {:?} }}",
            self.name,
            self.lib,
            self.params,
            self.return_type
        )
    }
}

#[cfg(test)]
impl PartialEq for ForeignFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.lib == other.lib && self.params == other.params &&
            self.return_type == other.return_type
    }
}

#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq, Debug))]
pub struct Foreign {
    pub tokens: Tokens,
    pub functions: Vec<ForeignFn>,
}

impl Foreign {
    pub fn new(tokens: Tokens, functions: Vec<ForeignFn>) -> Self {
        Foreign { tokens, functions }
    }
}

impl ASTNode for Foreign {
    fn tokens(&self) -> &[RcToken] {
        &self.tokens
    }

    fn lookaheads() -> &'static [TokenType] {
        lookaheads!(TokenType::KwForeign)
    }
}

#[cfg(test)]
impl Debug for Foreign {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Foreign {{ functions: {:?} }}", self.functions)
    }
}

#[cfg(test)]
impl PartialEq for Foreign {
    fn eq(&self, other: &Self) -> bool {
        self.functions == other.functions
    }
}

pub type TopLevelList = Vec<TopLevel>;

/// An unprocessed AST.
pub struct AST {
    pub ast: TopLevelList,
    pub path: String,
}

/*
/// A pre-processed AST, ready to be compiled.
pub struct FilledAST {
    pub ast: FunDefList,
    pub path: String,
}
*/
