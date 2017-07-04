use common::*;
use errors::*;
use syntax::token::*;
use syntax::ast::*;

pub struct Parser<'c> {
    tokenizer: Tokenizer<'c>,
    curr: Option<Token>,
}

impl<'c> Parser<'c> {
    pub fn new(tokenizer: Tokenizer<'c>) -> Self {
        let mut parser = Parser {
            tokenizer,
            curr: None,
        };
        parser.next_token().unwrap();
        parser
    }

    pub fn is_end(&self) -> bool {
        self.curr.is_none()
    }

    pub fn parse(&mut self) -> Result<AST> {
        unimplemented!()
    }

    fn match_token(&mut self, token_type: TokenType) -> Result<Token> {
        let curr = self.expect_curr()
            .chain_err(|| format!("expected token type `{}`", token_type))?
            .clone();
        if curr.token_type() == token_type {
            self.next_token()?;
            Ok(curr)
        }
        else {
            Err(format!("expected token type `{}`; instead got `{}`", token_type, curr.token_type()).into())
        }
    }

    fn match_any(&mut self, token_types: &[TokenType]) -> Result<Token> {
        let curr = self.expect_curr()
            .chain_err(|| format!("expected any token of type {}",
                                  token_types.iter()  // this ugly biz just makes prettier expected token types
                                      .map(|t| format!("`{}`", t))
                                      .collect::<Vec<_>>()
                                      .join(", "))
                       )?.clone();
        if token_types.contains(&curr.token_type()) {
            self.next_token()?;
            Ok(curr)
        }
        else {
            let expected_types = token_types.iter()
                .map(|t| format!("`{}`", t))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("expected any token of {}; instead got `{}`", expected_types, curr.token_type()).into())
        }
    }

    fn try_match_token(&mut self, token_type: TokenType) -> Result<Option<Token>> {
        if let Some(curr) = self.curr.clone() {
            if curr.token_type() == token_type {
                self.next_token()?;
                Ok(Some(curr))
            }
            else {
                Ok(None)
            }
        }
        else {
            Ok(None)
        }
    }

    fn try_match_any(&mut self, token_types: &[TokenType]) -> Result<Option<Token>> {
        if let Some(curr) = self.curr.clone() {
            if token_types.contains(&curr.token_type()) {
                self.next_token()?;
                Ok(Some(curr))
            }
            else {
                Ok(None)
            }
        }
        else {
            Ok(None)
        }
    }

    fn can_match_token(&self, token_type: TokenType) -> bool {
        self.curr
            .as_ref()
            .map(|t| t.token_type() == token_type)
            .unwrap_or(false)
    }

    fn can_match_any(&self, token_types: &[TokenType]) -> bool {
        self.curr
            .as_ref()
            .map(|t| token_types.contains(&t.token_type()))
            .unwrap_or(false)
    }

    fn expect_curr(&self) -> Result<&Token> {
        if let Some(ref curr) = self.curr {
            Ok(curr)
        }
        else {
            Err("unexpected EOF".into())
        }
    }

    fn next_token(&mut self) -> Result<()> {
        loop { 
            if let Some(result) = self.tokenizer.next() {
                let result = result?;
                // skip comments
                if result.token_type() != TokenType::Comment {
                    self.curr = Some(result);
                    break;
                }
            }
            else {
                self.curr = None;
                break;
            }
        }
        Ok(())
    }

    /*
     * Grammar rules
     */
    fn expect_stack_stmt(&mut self) -> Result<StackStmt> {
        let mut tokens = vec![];
        let mut actions = vec![];
        while !self.can_match_token(TokenType::Semi) && self.curr.is_some() {
            let action = self.expect_stack_action()?;
            tokens.extend_from_slice(action.tokens());
            actions.push(action);
        }
        tokens.push(self.match_token(TokenType::Semi)?.into_rc());
        Ok(StackStmt::new(tokens, actions))
    }

    fn expect_stack_action(&mut self) -> Result<StackAction> {
        if let Some(item) = self.try_item() {
            Ok(StackAction::Push(item))
        }
        else {
            let mut tokens = vec![self.match_token(TokenType::Dot)?.into_rc()];
            let item = self.expect_item()?;
            tokens.extend_from_slice(item.tokens());
            Ok(StackAction::Pop(tokens, item))
        }
    }

    fn expect_item(&mut self) -> Result<Item> {
        let token = self.match_any(Item::lookaheads())?;
        match token.token_type() {
            TokenType::LBrack => {
                let mut tokens = vec![token.into_rc()];
                // match a stack item
                let mut items = vec![];
                while !self.can_match_token(TokenType::RBrack) {
                    let item = self.expect_item()?;
                    tokens.extend_from_slice(item.tokens());
                    items.push(item);
                }
                tokens.push(self.match_token(TokenType::RBrack)?.into_rc());
                Ok(Item::new(tokens, ItemType::Stack(items)))
            },
            _ => Ok(token.into()),
        }
    }

    fn try_item(&mut self) -> Option<Item> {
        if self.can_match_any(Item::lookaheads()) {
            Some(self.expect_item().unwrap())
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use syntax::token::*;
    use syntax::parse::*;

    macro_rules! tests {
        ($text:expr, $(($($tt:tt)+))+) => {
            let t = Tokenizer::new("test", $text);
            let mut p = Parser::new(t);
            {
                $(check!(p, $($tt)+);)+
            }
            // skip trailing whitespace to square up the "end" mark
            assert!(p.is_end());
        };
    }

    macro_rules! check {
        ($parser:expr, $which:ident, $item:expr) => {
            let item = $parser.$which().unwrap();
            assert_eq!(item, $item);
        }
    }

    macro_rules! stack_stmt {
        (@ $which:ident Nil $($args:tt)*) => {{
            let mut v = stack_stmt!(@ $($args)*);
            v.insert(0, stack_action!($which Nil));
            v
        }};
        (@ $which:ident $i1:tt $i2:tt $($args:tt)*) => {{
            let mut v = stack_stmt!(@ $($args)*);
            v.insert(0, stack_action!($which $i1 $i2));
            v
        }};
        (@) => {
            vec![]
        };
        ($($args:tt)*) => {
            StackStmt::new(vec![], stack_stmt!(@ $($args)*))
        };
    }

    macro_rules! stack_action {
        (Push $($args:tt)+) => { StackAction::Push(item!($($args)+)) };
        (Pop $($args:tt)+) => { StackAction::Pop(vec![], item!($($args)+)) };
    }

    macro_rules! item {
        (Nil) => { Item::new(vec![], ItemType::Nil) };
        (Ident $value:expr) => { Item::new(vec![], ItemType::Ident($value.to_string())) };
        (String $value:expr) => { Item::new(vec![], ItemType::String($value.to_string())) };
        (Stack [$(($($args:tt)+))+]) => {
            Item::new(vec![], ItemType::Stack(
                        vec![ $(item!($($args)+)),+ ]
                    )
                )
        };
        ($type:ident $value:expr) => { Item::new(vec![], ItemType::$type($value)) };
    }

    #[test]
    fn test_parser_stack_stmts() {
        tests! {
            r#"
            1 2 3 .a .b .c;
            $ .@;
            ;
            @ [1 2 3 4 5] ;
            a .a b .foo c .bar d .x e .2 f .@ ;
            "#,
            (expect_stack_stmt, stack_stmt!(Push Int 1 Push Int 2 Push Int 3 Pop Ident "a" Pop Ident "b" Pop Ident "c"))
            (expect_stack_stmt, stack_stmt!(Push Ident "$" Pop Nil))
            (expect_stack_stmt, stack_stmt!())
            (expect_stack_stmt, stack_stmt!(Push Nil Push Stack [(Int 1) (Int 2) (Int 3) (Int 4) (Int 5)]))
            (expect_stack_stmt, stack_stmt!(
                    Push Ident "a"
                    Pop Ident "a"
                    Push Ident "b"
                    Pop Ident "foo"
                    Push Ident "c"
                    Pop Ident "bar"
                    Push Ident "d"
                    Pop Ident "x"
                    Push Ident "e"
                    Pop Int 2
                    Push Ident "f"
                    Pop Nil
            ))
        }
    }

    #[test]
    fn test_parser_stack_actions() {
        tests! {
            r#"
            a
            .a
            b
            .foo
            c
            .bar
            d
            .x
            e
            .2
            f
            .@
            "#,
            (expect_stack_action, stack_action!(Push Ident "a"))
            (expect_stack_action, stack_action!(Pop Ident "a"))
            (expect_stack_action, stack_action!(Push Ident "b"))
            (expect_stack_action, stack_action!(Pop Ident "foo"))
            (expect_stack_action, stack_action!(Push Ident "c"))
            (expect_stack_action, stack_action!(Pop Ident "bar"))
            (expect_stack_action, stack_action!(Push Ident "d"))
            (expect_stack_action, stack_action!(Pop Ident "x"))
            (expect_stack_action, stack_action!(Push Ident "e"))
            (expect_stack_action, stack_action!(Pop Int 2))
            (expect_stack_action, stack_action!(Push Ident "f"))
            (expect_stack_action, stack_action!(Pop Nil))
        };
    }

    #[test]
    fn test_parser_items() {
        tests! {
            r#"
            123456789
            987654321

            foo bar baz
            'a '\n '\s
            "This is\na string with \"escapes\""
            "this is a boring string without escapes"
            T F
            [ this is a stack ]
            @
            "#,

            (expect_item, item!(Int 123456789))
            (expect_item, item!(Int 987654321))
            (expect_item, item!(Ident "foo"))
            (expect_item, item!(Ident "bar"))
            (expect_item, item!(Ident "baz"))
            (expect_item, item!(Char 'a'))
            (expect_item, item!(Char '\n'))
            (expect_item, item!(Char ' '))
            (expect_item, item!(String "This is\na string with \"escapes\""))
            (expect_item, item!(String "this is a boring string without escapes"))
            (expect_item, item!(Bool true))
            (expect_item, item!(Bool false))
            (expect_item, item!(Stack [
                              (Ident "this")
                              (Ident "is")
                              (Ident "a")
                              (Ident "stack")
                            ])
             )
            (expect_item, item!(Nil))
        };
    }
}
