use prelude::*;
use std::str::Chars;
use std::rc::Rc;
use std::sync::Arc;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

lazy_static! {
    static ref ESCAPES: HashMap<char, char> = {
        hashmap! {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '0' => '\0',
            's' => ' ',
            '"' => '"',
            '\'' => '\'',
            '\\' => '\\',
        }
    };
    static ref ESCAPE_CHARS: String = {
        let mut s = String::new();
        for k in ESCAPES.keys() {
            s.push(*k);
        }
        s
    };

    static ref ESCAPE_VALS: String = {
        let mut s = String::new();
        for v in ESCAPES.values() {
            s.push(*v);
        }
        s
    };

}
const IDENT_CHARS: &str = "_!@$%^&|*-+/=<>abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
    Comment,

    // Varying things
    Int,
    BasedInt(usize),
    Float,
    String,
    Char,
    Ident,

    // Symbols
    Dot,
    Semi,
    LBrace,
    RBrace,
    LBrack,
    RBrack,

    // Keywords
    KwNil,
    KwImport,
    KwBr,
    KwElBr,
    KwEl,
    KwT,
    KwF,
    KwLoop,
    KwForeign,
    KwBake,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::TokenType::*;
        let s = match *self {
            Comment => "comment",
            Int => "int",
            BasedInt(_) => "int",
            Float => "float",
            String => "string",
            Char => "char",
            Ident => "ident",

            // Symbols
            Dot => "dot",
            Semi => "semicolon",
            LBrace => "lbrace",
            RBrace => "rbrace",
            LBrack => "lbrack",
            RBrack => "rbrack",

            // Keywords
            KwNil => "nil sigil",
            KwImport => "import keyword",
            KwBr => "br keyword",
            KwElBr => "elbr keyword",
            KwEl => "el keyword",
            KwT => "T keyword",
            KwF => "F keyword",
            KwLoop => "loop keyword",
            KwForeign => "foreign keyword",
            KwBake => "compile-time bake keyword",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    token_type: TokenType,
    range: Range,
}

pub type RcToken = Rc<Token>;

impl Token {
    pub fn new(token_type: TokenType, range: Range) -> Self {
        Token { token_type, range }
    }

    pub fn range(&self) -> Range {
        self.range.clone()
    }

    pub fn as_str(&self) -> &str {
        let ref range = self.range;
        let ref text = *range.start.source_text;
        let mut start = range.start.src_index as usize;
        let mut end = range.end.src_index as usize;
        if self.token_type == TokenType::String {
            start += 1;
            end -= 1;
        } else if self.token_type == TokenType::Char {
            start += 1;
        }
        assert!(start <= end);
        &text[start..end]
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn into_rc(self) -> Rc<Token> {
        Rc::new(self)
    }

    /// Unescapes the string returned from `self.as_str()`.
    pub fn unescape(&self) -> String {
        let s = self.as_str();
        if !s.contains('\\') {
            return s.to_string();
        }

        let mut built = String::with_capacity(s.len());
        let mut chars = s.chars();
        loop {
            if let Some(c) = chars.next() {
                if c == '\\' {
                    let c = chars.next().expect(
                        "TOKENIZER BUG: got escape start character, but no character following it",
                    );
                    let c = ESCAPES.get(&c)
                        .expect("TOKENIZER BUG: got invalid escape character that was not caught by the tokenizer");
                    built.push(*c);
                } else {
                    built.push(c);
                }
            } else {
                break;
            }
        }
        built
    }
}

impl PartialEq for Token {
    /// Implements comparison for tokens. Tokens are not equal by ranges; rather, they are equal by
    /// text content and token type.
    fn eq(&self, other: &Token) -> bool {
        self.token_type == other.token_type && self.as_str() == other.as_str()
    }
}

pub struct Tokenizer<'c> {
    curr: Option<char>,
    next: Option<char>,
    curr_range: Range,
    next_range: Range,
    source_chars: Chars<'c>,
    phantom_lifetime: PhantomData<&'c str>,
    source_path: RcStr,
    source_text: RcStr,
}

impl<'c> Tokenizer<'c> {
    pub fn new(source_path: &str, source_text: &'c str) -> Self {
        let rc_text = Arc::new(source_text.to_string());
        let rc_path = Arc::new(source_path.to_string());

        let mut t = Tokenizer {
            curr_range: Range::new_curr(rc_path.clone(), rc_text.clone()),
            next_range: Range::new_next(rc_path.clone(), rc_text.clone()),
            curr: None,
            next: None,
            source_chars: source_text.chars(),
            phantom_lifetime: PhantomData,
            source_path: rc_path,
            source_text: rc_text,
        };
        t.next_char();
        t.next_char(); // start it at the current character
        t
    }

    /// Gets whether the tokenizer has finished the source text.
    pub fn is_end(&self) -> bool {
        self.next.is_none() && self.curr.is_none()
    }

    /// Advances the tokenizer by one character.
    fn next_char(&mut self) -> Option<char> {
        let old = self.curr.clone();
        // update current token and position
        self.curr = self.next.clone();
        self.curr_range.adv();
        if let Some('\n') = old {
            self.curr_range.line();
        }

        // update next token and position
        self.next = self.source_chars.next();
        self.next_range.adv();
        if let Some('\n') = self.curr {
            self.next_range.line();
        }
        old
    }

    fn try_match_char(&mut self, c: char) -> bool {
        if let Some(curr) = self.curr {
            if curr == c {
                self.next_char();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn try_match_any(&mut self, char_list: &str) -> Option<char> {
        if let Some(curr) = self.curr {
            if char_list.contains(curr) {
                self.next_char()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn match_char(&mut self, c: char) -> Result<()> {
        if let Some(curr) = self.curr {
            if c == curr {
                self.next_char();
                Ok(())
            } else {
                Err(
                    format!("expected character `{}`; instead got `{}`", c, curr).into(),
                )
            }
        } else {
            Err(
                format!("expected character `{}`; instead got EOF", c).into(),
            )
        }
    }

    fn match_any_char(&mut self, char_list: &str) -> Result<()> {
        if let Some(curr) = self.curr {
            if char_list.contains(curr) {
                self.next_char();
                Ok(())
            } else {
                let expected_list = char_list
                    .chars()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(
                    format!(
                        "expected any of {}; instead got `{:?}`",
                        expected_list,
                        curr
                    ).into(),
                )
            }
        } else {
            let expected_list = char_list
                .chars()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
                .join(", ");
            Err(
                format!("expected any of {}; instead got EOF", expected_list).into(),
            )
        }
    }

    /// Matches any character *not* in the given arguments.
    /// This function still expects a character, so it will fail on EOF.
    fn match_any_char_except(&mut self, char_list: &str) -> Result<()> {
        if let Some(curr) = self.curr {
            if !char_list.contains(curr) {
                self.next_char();
                Ok(())
            } else {
                let expected_list = char_list
                    .split(|_| true)
                    .map(|c| format!("`{:?}`", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(
                    format!(
                        "expected any character EXCEPT {}; instead got `{:?}`",
                        expected_list,
                        curr
                    ).into(),
                )
            }
        } else {
            let expected_list = char_list
                .split(|_| true)
                .map(|c| format!("`{:?}`", c))
                .collect::<Vec<_>>()
                .join(", ");
            Err(
                format!(
                    "expected any character EXCEPT {}; instead got EOF",
                    expected_list
                ).into(),
            )
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.curr {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Attempts to match a comment.
    fn next_comment(&mut self) -> Result<Token> {
        self.match_char('#')?;
        if self.curr == Some('!') {
            self.next_char();
            while !(self.curr == Some('!') && self.next == Some('#')) {
                self.next_char();
            }
            self.match_char('!')?;
            self.match_char('#')?; // skip past the !#
        } else {
            while self.curr.is_some() && self.curr != Some('\n') {
                self.next_char();
            }
            self.match_char('\n')?; // skip past the newline
        }
        self.ok_token(TokenType::Comment)
    }

    /// Attempts to match an integer.
    fn next_int(&mut self) -> Result<Token> {
        const PREFICES: &[char] = &['x', 'X', 'b', 'B', 'o'];
        const DEC_DIGITS: &str = "0123456789";
        const HEX_DIGITS: &str = "0123456789abcdefABCDEF";
        const OCT_DIGITS: &str = "01234567";
        const BIN_DIGITS: &str = "01";

        self.try_match_char('-');

        if self.curr.map(|c| c == '0').unwrap_or(false) &&
            self.next.map(|c| PREFICES.contains(&c)).unwrap_or(false)
        {
            let (digits, base) = match self.next {
                // hex number
                Some('x') | Some('X') => {
                    self.next_char();
                    self.next_char();
                    (HEX_DIGITS, 16)
                }
                // binary number
                Some('b') | Some('B') => {
                    self.next_char();
                    self.next_char();
                    (BIN_DIGITS, 2)
                }
                // octal number
                Some('o') => {
                    self.next_char();
                    self.next_char();
                    (OCT_DIGITS, 8)
                }
                _ => unreachable!(),
            };
            self.match_any_char(digits)?;
            while let Some(_) = self.try_match_any(digits) {}
            self.ok_token(TokenType::BasedInt(base))
        } else {
            self.match_any_char(DEC_DIGITS)?;
            while let Some(_) = self.try_match_any(DEC_DIGITS) {}
            self.ok_token(TokenType::Int)
        }
    }

    /// Attempts to match a string.
    fn next_string(&mut self) -> Result<Token> {
        self.match_char('"')?;
        while !self.try_match_char('"') {
            if let Some(curr) = self.curr {
                if curr == '\\' {
                    self.next_char();
                    // match escapes
                    self.match_any_char(&ESCAPE_CHARS)?;
                } else {
                    self.next_char();
                }
            } else {
                return Err(
                    "expected string character or `\"` character; instead got EOF".into(),
                );
            }
        }
        self.ok_token(TokenType::String)
    }

    /// Attempts to match a character.
    fn next_character(&mut self) -> Result<Token> {
        self.match_char('\'')?;
        if self.try_match_char('\\') {
            self.match_any_char(&ESCAPE_CHARS)?;
        } else {
            self.match_any_char_except(&ESCAPE_VALS)?;
        }
        self.ok_token(TokenType::Char)
    }

    fn next_identifier(&mut self) -> Result<Token> {
        lazy_static! {
            static ref KEYWORDS: HashMap<&'static str, TokenType> = {
                hashmap! {
                    "br" => TokenType::KwBr,
                    "elbr" => TokenType::KwElBr,
                    "el" => TokenType::KwEl,
                    "loop" => TokenType::KwLoop,
                    "@" => TokenType::KwNil,
                    "import" => TokenType::KwImport,
                    "T" => TokenType::KwT,
                    "F" => TokenType::KwF,
                    "foreign" => TokenType::KwForeign,
                    "bake" => TokenType::KwBake,
                }
            };
        };
        self.match_any_char(IDENT_CHARS)?;
        while self.try_match_any(IDENT_CHARS).is_some() {}
        if let Some(ty) = KEYWORDS.get(self.curr_range.as_str()) {
            self.ok_token(*ty)
        } else {
            self.ok_token(TokenType::Ident)
        }
    }

    fn match_single_token(&mut self, c: char, token_type: TokenType) -> Result<Token> {
        self.match_char(c)?;
        self.ok_token(token_type)
    }

    /// Convenience function to make new tokens
    fn ok_token(&mut self, token_type: TokenType) -> Result<Token> {
        Ok(Token::new(token_type, self.curr_range.clone()))
    }

    pub fn source_path(&self) -> RcStr {
        self.source_path.clone()
    }

    pub fn source_text(&self) -> RcStr {
        self.source_text.clone()
    }
}

impl<'c> Iterator for Tokenizer<'c> {
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Result<Token>> {
        self.skip_ws();

        if self.is_end() {
            return None;
        }
        assert!(self.curr.is_some());
        self.curr_range.catchup();
        self.next_range.catchup();

        match self.curr.unwrap() {
            // comment
            '#' => Some(self.next_comment()),
            // negative integer
            '-' if self.next.map(|c| c.is_digit(10)).unwrap_or(false) => Some(self.next_int()),
            // positive integer
            '0'...'9' => Some(self.next_int()),
            // string
            '"' => Some(self.next_string()),
            // char
            '\'' => Some(self.next_character()),
            // dot
            '.' => Some(self.match_single_token('.', TokenType::Dot)),
            // lbrace
            '{' => Some(self.match_single_token('{', TokenType::LBrace)),
            // rbrace
            '}' => Some(self.match_single_token('}', TokenType::RBrace)),
            // lbrack
            '[' => Some(self.match_single_token('[', TokenType::LBrack)),
            // rbrack
            ']' => Some(self.match_single_token(']', TokenType::RBrack)),
            // semicolon
            ';' => Some(self.match_single_token(';', TokenType::Semi)),
            // try for an identifier
            _ => Some(self.next_identifier()),
        }
    }
}

#[cfg(test)]
mod test {
    use syntax::token::*;

    macro_rules! tests {
        ($text:expr, $(($($tt:tt)+))+) => {
            let mut t = Tokenizer::new("test", $text);
            {
                $(check!(t, $($tt)+);)+
            }
            // skip trailing whitespace to square up the "end" mark
            t.skip_ws();
            assert!(t.is_end());
        };
    }

    macro_rules! check {
        ($t:expr, $type:expr) => {
            let next = $t.next()
                .expect("expected token; instead got EOF")
                .expect("expected token; instead got error");
            assert_eq!(next.token_type(), $type);
        };

        ($t:expr, $type:expr, $text:expr) => {
            let next = $t.next()
                .expect("expected token; instead got EOF")
                .expect("expected token; instead got error");
            assert_eq!(next.token_type(), $type);
            assert_eq!(next.as_str(), $text);
        }; 
    }

    #[test]
    fn test_lexer_keywords() {
        tests! {
            r#"
            br
            elbr
            el
            loop
            T
            F
            @
            import
            foreign
            "#,
            (TokenType::KwBr)
            (TokenType::KwElBr)
            (TokenType::KwEl)
            (TokenType::KwLoop)
            (TokenType::KwT)
            (TokenType::KwF)
            (TokenType::KwNil)
            (TokenType::KwImport)
            (TokenType::KwForeign)
        };
    }

    #[test]
    fn test_lexer_idents() {
        tests! {
            r#"
            foo
            bar
            baz
            v-a-l-u-e
            t
            f
            ==
            !=
            <=
            >=
            ********
            "#,

            (TokenType::Ident, "foo")
            (TokenType::Ident, "bar")
            (TokenType::Ident, "baz")
            (TokenType::Ident, "v-a-l-u-e")
            (TokenType::Ident, "t")
            (TokenType::Ident, "f")
            (TokenType::Ident, "==")
            (TokenType::Ident, "!=")
            (TokenType::Ident, "<=")
            (TokenType::Ident, ">=")
            (TokenType::Ident, "********")
        };
    }

    #[test]
    fn test_lexer_syms() {
        tests! {
            r#"
            . [ ] { } ;
            "#,
            (TokenType::Dot)
            (TokenType::LBrack)
            (TokenType::RBrack)
            (TokenType::LBrace)
            (TokenType::RBrace)
            (TokenType::Semi)
        };
    }

    #[test]
    fn test_lexer_chars() {
        tests! {
            r#"
            'a
            'b
            'c
            'd
            '\s
            '\n
            '\t
            '\\
            '\'
            '\"
            "#,

            (TokenType::Char, "a")
            (TokenType::Char, "b")
            (TokenType::Char, "c")
            (TokenType::Char, "d")
            (TokenType::Char, "\\s")
            (TokenType::Char, "\\n")
            (TokenType::Char, "\\t")
            (TokenType::Char, "\\\\")
            (TokenType::Char, "\\'")
            (TokenType::Char, "\\\"")
        };
    }

    #[test]
    fn test_lexer_strings() {
        tests! {
            r#"
            "the quick brown fox jumped over the lazy dogs"
            "the\squick \"brown\" fox \'jumped\' over the lazy dogs"
            "the quick brown fox\r\njumped over the lazy dogs\0"
            "#,

            (TokenType::String, "the quick brown fox jumped over the lazy dogs")
            (TokenType::String, "the\\squick \\\"brown\\\" fox \\'jumped\\' over the lazy dogs")
            (TokenType::String, "the quick brown fox\\r\\njumped over the lazy dogs\\0")
        };
    }

    #[test]
    fn test_lexer_ints() {
        tests! {
            r#"
            0
            12
            34
            123456789
            1100010010
            0o77447744
            0xFF77FF88
            0b11001010
            -0
            -12
            -34
            -123456789
            -1100010010
            -0o77447744
            -0xFF77FF88
            -0b11001010
            "#,

            (TokenType::Int, "0")
            (TokenType::Int, "12")
            (TokenType::Int, "34")
            (TokenType::Int, "123456789")
            (TokenType::Int, "1100010010")
            (TokenType::BasedInt(8), "0o77447744")
            (TokenType::BasedInt(16), "0xFF77FF88")
            (TokenType::BasedInt(2), "0b11001010")
            (TokenType::Int, "-0")
            (TokenType::Int, "-12")
            (TokenType::Int, "-34")
            (TokenType::Int, "-123456789")
            (TokenType::Int, "-1100010010")
            (TokenType::BasedInt(8), "-0o77447744")
            (TokenType::BasedInt(16), "-0xFF77FF88")
            (TokenType::BasedInt(2), "-0b11001010")
        };
    }

    #[test]
    fn test_lexer_comments() {
        tests! {
            r#"
            # this is a comment
            # this is a comment, too
            # foo, bar, baz
            #! this is a comment !#
            #! this is a comment too !#
            #!
            * foo
            * bar
            * baz
            !#"#,

            (TokenType::Comment)
            (TokenType::Comment)
            (TokenType::Comment)
            (TokenType::Comment)
            (TokenType::Comment)
            (TokenType::Comment)
        };
    }
}
