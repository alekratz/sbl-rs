use prelude::*;

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

    pub fn parse(&mut self) -> Result<TopLevelList> {
        let mut ast = TopLevelList::new();
        while !self.is_end() {
            let top_level = self.expect_top_level();
            if top_level.is_err() {
                // get the range of the most recent token
                let ref tokenizer = self.tokenizer;
                let curr_range = self.curr.as_ref().map(Token::range).unwrap_or(Range::eof(
                    tokenizer.source_path(),
                    tokenizer.source_text(),
                ));
                top_level.chain_err(|| curr_range)?;
            } else {
                ast.push(top_level.unwrap())
            }
        }
        Ok(ast)
    }

    fn match_token(&mut self, token_type: TokenType) -> Result<Token> {
        let curr = self.expect_curr()
            .chain_err(|| format!("expected token type `{}`", token_type))?
            .clone();
        if curr.token_type() == token_type {
            self.next_token()?;
            Ok(curr)
        } else {
            Err(
                format!(
                    "expected token type `{}`; got `{}` instead",
                    token_type,
                    curr.token_type()
                ).into(),
            )
        }
    }

    fn match_any(&mut self, token_types: &[TokenType]) -> Result<Token> {
        let curr = self.expect_curr()
            .chain_err(|| {
                format!(
                    "expected any token of type {}",
                    token_types.iter()  // this ugly biz just makes prettier expected token types
                                      .map(|t| format!("`{}`", t))
                                      .collect::<Vec<_>>()
                                      .join(", ")
                )
            })?
            .clone();
        if token_types.contains(&curr.token_type()) {
            self.next_token()?;
            Ok(curr)
        } else {
            let mut expected_types = token_types
                .iter()
                .map(|t| format!("`{}`", t))
                .collect::<Vec<_>>();
            expected_types.sort();
            expected_types.dedup();
            Err(
                format!(
                    "expected any token of {}; got `{}` instead",
                    expected_types.join(", "),
                    curr.token_type()
                ).into(),
            )
        }
    }

    fn can_match_token(&self, token_type: TokenType) -> bool {
        self.curr
            .as_ref()
            .map(|t| t.token_type() == token_type)
            .unwrap_or(false)
    }

    fn can_match_any(&self, token_types: &[TokenType]) -> bool {
        let result = self.curr
            .as_ref()
            .map(|t| token_types.contains(&t.token_type()))
            .unwrap_or(false);
        result
    }

    fn expect_curr(&self) -> Result<&Token> {
        if let Some(ref curr) = self.curr {
            Ok(curr)
        } else {
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
            } else {
                self.curr = None;
                break;
            }
        }
        Ok(())
    }

    /*
     * Grammar rules
     */
    fn expect_top_level(&mut self) -> Result<TopLevel> {
        if self.can_match_any(Import::lookaheads()) {
            Ok(TopLevel::Import(self.expect_import().chain_err(
                || "while parsing import statement",
            )?))
        } else if self.can_match_any(BCFunDef::lookaheads()) {
            Ok(TopLevel::BCFunDef(self.expect_fun()?))
        } else if self.can_match_any(Foreign::lookaheads()) {
            Ok(TopLevel::Foreign(self.expect_foreign()?))
        } else {
            let mut all = vec![];
            all.extend_from_slice(BCFunDef::lookaheads());
            all.extend_from_slice(Import::lookaheads());
            all.extend_from_slice(Foreign::lookaheads());
            self.match_any(all.as_slice())?;
            unreachable!()
        }
    }

    fn expect_import(&mut self) -> Result<Import> {
        let mut tokens = vec![self.match_any(Import::lookaheads())?.into_rc()];
        let str_token = self.match_token(TokenType::String)?;
        let path = str_token.unescape();
        tokens.push(str_token.into_rc());
        Ok(Import::new(tokens, path))
    }

    fn expect_foreign(&mut self) -> Result<Foreign> {
        let mut tokens = vec![self.match_any(Foreign::lookaheads())?.into_rc()];
        let lib_token = self.match_token(TokenType::String)?;
        let lib = lib_token.unescape();
        tokens.push(lib_token.into_rc());
        // TODO : allow for single-shot function definitions that don't require braces that don't
        // require braces
        tokens.push(self.match_token(TokenType::LBrace)?.into_rc());
        let mut funs = vec![];
        while !self.can_match_token(TokenType::RBrace) && self.curr.is_some() {
            let foreign_fn = self.expect_foreign_fun(&lib)?;
            tokens.append_node(&foreign_fn);
            funs.push(foreign_fn);
        }
        tokens.push(self.match_token(TokenType::RBrace)?.into_rc());
        Ok(Foreign::new(tokens, funs))
    }

    fn expect_foreign_fun(&mut self, lib: &str) -> Result<ForeignFun> {
        fn type_map(name: &str) -> Result<ItemType> {
            match name {
                "int" => Ok(ItemType::Int(0)),
                "char" => Ok(ItemType::Char('\0')),
                "string" => Ok(ItemType::String(String::new())),
                "bool" => Ok(ItemType::Bool(false)),
                "void" => Ok(ItemType::Nil),
                t => Err(format!("unknown foreign type `{}`", t).into()),
            }
        }

        let return_type_token = self.match_token(TokenType::Ident)?;
        let return_type = type_map(return_type_token.as_str())?;

        let name_token = self.match_token(TokenType::Ident)?;
        let name = String::from(name_token.as_str());

        let mut tokens = vec![
            return_type_token.into_rc(),
            name_token.into_rc(),
            self.match_token(TokenType::LBrack)?.into_rc(),
        ];
        let mut params = vec![];
        // go through all of the types
        while !self.can_match_token(TokenType::RBrack) && self.curr.is_some() {
            let param_token = self.match_token(TokenType::Ident)?;
            params.push(type_map(param_token.as_str())?);
            tokens.push(param_token.into_rc());
        }
        tokens.push(self.match_token(TokenType::RBrack)?.into_rc());
        Ok(ForeignFun::new(
            tokens,
            name,
            lib.to_string(),
            params,
            return_type,
        ))
    }

    fn expect_fun(&mut self) -> Result<BCFunDef> {
        let mut tokens = vec![self.match_any(BCFunDef::lookaheads())?.into_rc()];
        let name = tokens[0].as_str().to_string();
        let block = self.expect_block().chain_err(|| {
            format!("while parsing function `{}`", name)
        })?;
        tokens.append_node(&block);
        Ok(BCFunDef::new(tokens, name, block))
    }

    fn expect_stmt(&mut self) -> Result<Stmt> {
        if self.can_match_any(BrStmt::lookaheads()) {
            Ok(Stmt::Br(self.expect_br_stmt()?))
        } else if self.can_match_any(LoopStmt::lookaheads()) {
            Ok(Stmt::Loop(self.expect_loop_stmt()?))
        } else if self.can_match_any(StackStmt::lookaheads()) {
            Ok(Stmt::Stack(self.expect_stack_stmt()?))
        } else if self.can_match_any(BakeStmt::lookaheads()) {
            Ok(Stmt::Bake(self.expect_bake_stmt()?))
        } else {
            self.match_any(Stmt::lookaheads())?;
            unreachable!()
        }
    }

    fn expect_block(&mut self) -> Result<Block> {
        let mut tokens = vec![self.match_any(Block::lookaheads())?.into_rc()];
        let mut block = vec![];
        while !self.can_match_token(TokenType::RBrace) && self.curr.is_some() {
            let stmt = self.expect_stmt()?;
            tokens.append_node(&stmt);
            block.push(stmt);
        }
        tokens.push(self.match_token(TokenType::RBrace)?.into_rc());
        Ok(Block::new(tokens, block))
    }

    fn expect_block_actions(&mut self) -> Result<BlockActions> {
        let mut tokens = vec![];
        let mut actions = vec![];
        while !self.can_match_token(TokenType::LBrace) {
            let action = if tokens.len() > 0 {
                self.expect_stack_action().chain_err(|| tokens.range())
            } else {
                self.expect_stack_action()
            }?;
            tokens.append_node(&action);
            actions.push(action);
        }
        Ok(BlockActions::new(tokens, actions))
    }

    fn expect_loop_stmt(&mut self) -> Result<LoopStmt> {
        let mut tokens = vec![self.match_any(LoopStmt::lookaheads())?.into_rc()];
        let actions = self.expect_block_actions()?;
        tokens.append_node(&actions);
        let block = self.expect_block()?;
        tokens.append_node(&block);
        Ok(LoopStmt::new(tokens, actions, block))
    }

    fn expect_br_stmt(&mut self) -> Result<BrStmt> {
        let mut tokens = vec![self.match_any(BrStmt::lookaheads())?.into_rc()];
        let actions = self.expect_block_actions()?;
        tokens.append_node(&actions);
        let block = self.expect_block()?;
        tokens.append_node(&block);
        let mut elbr_stmts = vec![];
        while self.can_match_any(ElBrStmt::lookaheads()) {
            elbr_stmts.push(self.expect_elbr_stmt()?);
        }
        elbr_stmts.iter()
            .for_each(|s| tokens.append_node(s));
        let el_stmt = if self.can_match_any(ElStmt::lookaheads()) {
            let el_stmt = self.expect_el_stmt()?;
            tokens.append_node(&el_stmt);
            Some(el_stmt)
        } else {
            None
        };
        Ok(BrStmt::new(tokens, actions, block, elbr_stmts, el_stmt))
    }

    fn expect_elbr_stmt(&mut self) -> Result<ElBrStmt> {
        let mut tokens = vec![self.match_any(ElBrStmt::lookaheads())?.into_rc()];
        let actions = self.expect_block_actions()?;
        tokens.append_node(&actions);
        let block = self.expect_block()?;
        tokens.append_node(&block);
        Ok(ElBrStmt::new(tokens, actions, block))
    }

    fn expect_el_stmt(&mut self) -> Result<ElStmt> {
        let mut tokens = vec![self.match_any(ElStmt::lookaheads())?.into_rc()];
        let block = self.expect_block()?;
        tokens.append_node(&block);
        Ok(ElStmt::new(tokens, block))
    }

    fn expect_bake_stmt(&mut self) -> Result<BakeStmt> {
        let mut tokens = vec![self.match_any(BakeStmt::lookaheads())?.into_rc()];
        let block = self.expect_block()?;
        tokens.append_node(&block);
        Ok(BakeStmt::new(tokens, block))
    }

    fn expect_stack_stmt(&mut self) -> Result<StackStmt> {
        let mut tokens = vec![];
        let mut actions = vec![];
        while !self.can_match_any(
            &[
                TokenType::RBrace,
                TokenType::KwBr,
                TokenType::KwLoop,
                TokenType::KwBake,
            ],
        ) && self.curr.is_some()
        {
            let action = if tokens.len() > 0 {
                self.expect_stack_action().chain_err(|| tokens.range())
            } else {
                self.expect_stack_action()
            }?;
            tokens.append_node(&action);
            actions.push(action);
        }
        Ok(StackStmt::new(tokens, actions))
    }

    fn expect_stack_action(&mut self) -> Result<StackAction> {
        if let Some(item) = self.try_item() {
            Ok(StackAction::Push(item))
        } else {
            let mut tokens = vec![self.match_any(StackAction::lookaheads())?.into_rc()];
            let item = self.expect_item()?;
            tokens.append_node(&item);
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
                    tokens.append_node(&item);
                    items.push(item);
                }
                tokens.push(self.match_token(TokenType::RBrack)?.into_rc());
                Ok(Item::new(tokens, ItemType::Stack(items)))
            }
            _ => Ok(token.into()),
        }
    }

    fn try_item(&mut self) -> Option<Item> {
        // TODO : backtrack on failure
        if self.can_match_any(Item::lookaheads()) {
            Some(self.expect_item().unwrap())
        } else {
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

    // AST Items
    macro_rules! top_level {
        (BCFunDef $($tail:tt)+) => { TopLevel::BCFunDef(fun!($($tail)+)) };
        (Import $($tail:tt)+) => { TopLevel::Import(import!($($tail)+)) };
        (Foreign $($tail:tt)+) => { TopLevel::Foreign(foreign!($($tail)+)) };
    }

    macro_rules! fun {
        ($name:expr => { $($tail:tt)* }) => { BCFunDef::new(vec![], $name.to_string(), block!($($tail)*)) };
    }

    macro_rules! import {
        ($path:expr) => { Import::new(vec![], $path.to_string()) };
    }

    macro_rules! foreign {
        ($path:expr ; { $($tail:tt)* }) => {
            Foreign::new(vec![], foreign_fn!($path ; $($tail)*) )
        };
    }

    macro_rules! foreign_fn {
        ($path:expr ; $type:ident $name:ident [ $($params:ident)* ] $($tail:tt)*) => {{
            let mut v = vec![
                ForeignFun::new(vec![], stringify!($name).to_string(), $path.to_string(), vec![$(param!($params)),*], param!($type))
            ];
            v.append(&mut foreign_fn!($path ; $($tail)*));
            v
        }};
        ($path:expr ;) => { vec![] };
    }

    macro_rules! param {
        (int) => { ItemType::Int(0) };
        (char) => { ItemType::Char('\0') };
        (string) => { ItemType::String(String::new()) };
        (bool) => { ItemType::Bool(false) };
        (void) => { ItemType::Nil };
    }

    macro_rules! stmt {
        (Stack $($tail:tt)* ) => { Stmt::Stack(stack_stmt!($($tail)*)) };
        (Br ( $($actions:tt)* ) { $($tail:tt)* } ) =>
            { Stmt::Br(br_stmt!(($($actions)*), ($($tail)*) )) };
        (Br ( $($actions:tt)* ) { $($br_tail:tt)* } $(ElBr ( $($elbr_actions:tt)* ) { $($elbr_tail:tt)* })* El { $($el_tail:tt)* } ) =>
            { Stmt::Br(br_stmt!(($($actions)*), ($($br_tail)*), $(( ElBr ($($elbr_actions)*), ($($elbr_tail)*)),)* (El $($el_tail)*))) };
        (Loop ( $($actions:tt)* ) { $($tail:tt)* } ) => { Stmt::Loop(loop_stmt!(($($actions)*), ($($tail)*))) };
    }

    macro_rules! block_actions {
        ( $($actions:tt)* ) => {
            BlockActions::from(stack_stmt!($($actions)*))
        };
    }

    macro_rules! block {
        ($(($($args:tt)+))*) => {
            Block::new(vec![], vec![ $( stmt!($($args)+) ),* ])
        }
    }

    macro_rules! loop_stmt {
        ( ($($actions:tt)*), ($($block:tt)*) ) => {
            LoopStmt::new(vec![], block_actions!($($actions)*), block!($( $block )*))
        }
    }

    macro_rules! br_stmt {
        ( ( $($actions:tt)* ), ( $($br_args:tt)* ), $(( ElBr $($elbr_args:tt)+ ),)* ( El $($el_args:tt)* ) ) => {
            BrStmt::new(
                vec![],
                block_actions!($($actions)*),
                block!($($br_args)*),
                vec![ $( elbr_stmt!($($elbr_args)+) ),* ],
                Some(el_stmt!($($el_args)*)))
        };

        ( ( $($actions:tt)* ), ( $($br_args:tt)* ) ) => {
            BrStmt::new(
                vec![],
                block_actions!($($actions)*),
                block!($($br_args)*),
                vec![],
                None)
        };
    }

    macro_rules! elbr_stmt {
        ( ( $($actions:tt)* ), ($($block:tt)*) ) => { ElBrStmt::new(vec![], block_actions!($($actions)*), block!($($block)*)) };
    }

    macro_rules! el_stmt {
        ( $($args:tt)* ) => {
            ElStmt::new(vec![], block!($($args)*))
        };
    }

    macro_rules! stack_stmt {
        ($($args:tt)*) => {
            StackStmt::new(vec![], stack_stmt_tail!(@ $($args)*))
        };
    }

    macro_rules! stack_stmt_tail {
        (@ $which:ident Nil $($args:tt)*) => {{
            let mut v = stack_stmt_tail!(@ $($args)*);
            v.insert(0, stack_action!($which Nil));
            v
        }};
        (@ $which:ident $i1:tt $i2:tt $($args:tt)*) => {{
            let mut v = stack_stmt_tail!(@ $($args)*);
            v.insert(0, stack_action!($which $i1 $i2));
            v
        }};
        (@) => {
            vec![]
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
    fn test_parser_ast() {
        tests! {
            r#"
            import "test.sbl"
            import "basic.sbl"

            foreign "libc.6.so" {
                int open [ string string ]
                int close [ int ]
            }

            foo {
                1 2 3 .a .b .c
                $ .@
                @ [1 2 3 4 5]
                @ [ 0x10 0b10 0o10 ]
            }

            main {
                a .a b .foo c .bar d .x e .2 f .@
                loop {
                    .@
                    pop ^ println 0 ==
                }
                br {
                    "success" println
                    br msg {
                        "success message: " print println
                    }
                }
                elbr partial_success {
                    "partial success" println
                }
                el {
                    "failure:" println
                    loop {
                        "\t" print println
                    }
                }
            }
            "#,

            (expect_top_level, top_level!(Import "test.sbl"))
            (expect_top_level, top_level!(Import "basic.sbl"))
            (expect_top_level, top_level!(Foreign "libc.6.so" ; {
                int open [ string string ]
                int close [ int ]
            }))
            (expect_top_level, top_level!(BCFunDef "foo" => {
                (Stack Push Int 1 Push Int 2 Push Int 3 Pop Ident "a" Pop Ident "b" Pop Ident "c"
                       Push Ident "$" Pop Nil
                       Push Nil Push Stack [(Int 1) (Int 2) (Int 3) (Int 4) (Int 5)]
                       Push Nil Push Stack [(Int 16) (Int 2) (Int 8)])
            }))
            (expect_top_level, top_level!(BCFunDef "main" => {
                (Stack
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
                    Pop Nil)
                (Loop () {
                    (Stack Pop Nil
                           Push Ident "pop" Push Ident "^" Push Ident "println" Push Int 0 Push Ident "==")
                })
                (Br () {
                    (Stack Push String "success" Push Ident "println")
                    (Br (Push Ident "msg") {
                        (Stack Push String "success message: " Push Ident "print" Push Ident "println")
                    })
                }
                ElBr (Push Ident "partial_success") {
                    (Stack Push String "partial success" Push Ident "println")
                }
                El {
                    (Stack Push String "failure:" Push Ident "println")
                    (Loop () {
                        (Stack Push String "\t" Push Ident "print" Push Ident "println")
                    })
                })
            }))
        };
    }

    #[test]
    fn test_parser_stmts() {
        tests! {
            r#"
            1 2 3 .a .b .c
            $ .@

            @ [1 2 3 4 5]
            a .a b .foo c .bar d .x e .2 f .@
            loop 0 == {
                pop ^ println
            }
            br {
                "success" println
                br {
                    "success message: " print println
                }
            }
            el {
                "failure:" println
                loop {
                    "\t" print println
                }
            }
            "#,
            (expect_stmt, stmt!(Stack Push Int 1 Push Int 2 Push Int 3 Pop Ident "a" Pop Ident "b" Pop Ident "c"
                    Push Ident "$" Pop Nil
                    Push Nil Push Stack [(Int 1) (Int 2) (Int 3) (Int 4) (Int 5)]
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
            (expect_stmt, stmt!(Loop (Push Int 0 Push Ident "==") {
                (Stack Push Ident "pop" Push Ident "^" Push Ident "println")
            }))
            (expect_stmt, stmt!(
                Br () {
                    (Stack Push String "success" Push Ident "println")
                    (Br () {
                        (Stack Push String "success message: " Push Ident "print" Push Ident "println")
                    })
                }
                El {
                    (Stack Push String "failure:" Push Ident "println")
                    (Loop () {
                        (Stack Push String "\t" Push Ident "print" Push Ident "println")
                    })
                }
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
