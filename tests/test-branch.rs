extern crate sbl;
use sbl::prelude::*;

/// A macro that generates tests for a code string, and compares its resultant stack with a
/// specified vector to ensure that they match.
macro_rules! state_test {
    ($code:expr, $expected:expr) => {{
        let test_code = $code;
        let tokenizer = Tokenizer::new("test", &test_code);
        let mut parser = Parser::new(tokenizer);
        let ast = AST {
            ast: parser.parse().expect("Parse error"),
            path: "test".into(),
        }.preprocess::<&str>(&[]).expect("Preprocess error");
        let ir_compiler = CompileIR::new(&ast);
        let bc_compiler = CompileBytes::new(ir_compiler.compile().expect("IR compile error"));
        let fun_table = bc_compiler.compile().expect("BC compile error");
        let mut vm = VM::new(fun_table);
        vm.run().expect("Runtime error");
        assert_eq!(vm.state().stack, $expected);
    }}
}
// TODO
// Have this macro also run optimizations, because we want to know immediately when an optimization breaks current implementations
// TODO


#[test]
fn test_solo_br() {
    // Rudimentary branch tests
    state_test!(r#"main { br T { 1292 } }"#, vec![BCVal::Int(1292)]);
    state_test!(r#"main { br T { "asdf" } }"#, vec![BCVal::String("asdf".into())]);
    state_test!(r#"main { br T { '0 } }"#, vec![BCVal::Char('0')]);
    state_test!(r#"main { br T { } }"#, vec![]);
    state_test!(r#"main { br F { 9999 9999 9999 999 9 9 9 9 9 9 99999 } }"#, vec![]);
    // More complicated branch tests
    // TODO : below causes error because builtins weren't loaded -- compiler should catch this -- find this bug
    //state_test!(r#"main { br 5 5 == { 9999 } }"#, vec![BCVal::Int(9999)]);
    //state_test!(r#"main { br "asdf" "asdf" == { 9999 } }"#, vec![BCVal::Int(9999)]);
    state_test!(r#"main { T br { 9999 } }"#, vec![BCVal::Int(9999)]);
    state_test!(r#"main { br F { 9999 } br T { 55555 } }"#, vec![BCVal::Int(55555)]);
    state_test!(r#"main { br 0 { 9999 } }"#, vec![BCVal::Int(9999)]);
    // Nested statements
    state_test!(r#"main { br T { br T { 55555 } } }"#, vec![BCVal::Int(55555)]);
    state_test!(r#"main { T br { T br { 9999 } br T { 55555 } } }"#, vec![BCVal::Int(9999), BCVal::Int(55555)]);
    state_test!(r#"main { br 9999 55555 { } }"#, vec![BCVal::Int(9999)]);
}

#[test]
fn test_br_el() {
    // Rudimentary else tests
    state_test!(r#"main { br F { 8888 } el { 9999 } }"#, vec![BCVal::Int(9999)]);
    state_test!(r#"main { br F { 8888 } el { 9999 7777 } }"#, vec![BCVal::Int(9999), BCVal::Int(7777)]);
    state_test!(r#"main { br T { 8888 } el { 9999 } }"#, vec![BCVal::Int(8888)]);
    state_test!(r#"main { br T { 8888 } el { 9999 7777 } }"#, vec![BCVal::Int(8888)]);
    state_test!(r#"main { br T { 8888 } el { 9999 } 7777 }"#, vec![BCVal::Int(8888), BCVal::Int(7777)]);
    // Nested statements
    state_test!(r#"main { br F { 8888 } el { br 6666 F { 7777 } el { 9999 } } } "#, vec![BCVal::Int(6666), BCVal::Int(9999)]);
    state_test!(r#"main { br 6666 F { 8888 } el { br F { 7777 } el { 9999 } } } "#, vec![BCVal::Int(6666), BCVal::Int(9999)]);
}

#[test]
fn test_br_elbr() {
}

#[test]
fn test_br_elbr_el() {
}
