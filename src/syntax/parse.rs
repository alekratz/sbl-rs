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

    fn next_item(&mut self) -> Result<Item> {
        let token = self.match_any(Item::lookaheads())?;
        match token.token_type() {
            TokenType::LBrack => {
                let mut tokens = vec![token.into_rc()];
                // match a stack item
                let mut items = vec![];
                while !self.can_match_token(TokenType::RBrack) {
                    let item = self.next_item()?;
                    tokens.extend_from_slice(item.tokens());
                    items.push(item);
                }
                tokens.push(self.match_token(TokenType::RBrack)?.into_rc());
                Ok(Item::new(tokens, ItemType::Stack(items)))
            },
            _ => Ok(token.into()),
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

    macro_rules! item {
        (Nil) => { Item::new(vec![], ItemType::Nil) };
        (Ident, $value:expr) => { Item::new(vec![], ItemType::Ident($value.to_string())) };
        (String, $value:expr) => { Item::new(vec![], ItemType::String($value.to_string())) };
        ($type:ident, $value:expr) => { Item::new(vec![], ItemType::$type($value)) };
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

            (next_item, item!(Int, 123456789))
            (next_item, item!(Int, 987654321))
            (next_item, item!(Ident, "foo"))
            (next_item, item!(Ident, "bar"))
            (next_item, item!(Ident, "baz"))
            (next_item, item!(Char, 'a'))
            (next_item, item!(Char, '\n'))
            (next_item, item!(Char, ' '))
            (next_item, item!(String, "This is\na string with \"escapes\""))
            (next_item, item!(String, "this is a boring string without escapes"))
            (next_item, item!(Bool, true))
            (next_item, item!(Bool, false))
            (next_item, item!(Stack, vec![
                              item!(Ident, "this"), item!(Ident, "is"),
                              item!(Ident, "a"), item!(Ident, "stack")]))
            (next_item, item!(Nil))
        };
    }
}
