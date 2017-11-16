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
        let state: State = vm.into();
        assert_eq!(state.stack, $expected);
    }}
}
// TODO
// Have this macro also run optimizations, because we want to know immediately when an optimization breaks current implementations
// TODO


#[test]
fn test_solo_br() {
    state_test!(r#"main { br 1111 { br T { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 { br T { 2222 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { br F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br 1111 { br F { 2222 } 8765 } }"#, vec![BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { br T F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 { br T F { 2222 } 8765 } }"#, vec![BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } } }"#, vec![BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br F { 2222 } } }"#, vec![BCVal::Int(5678)]);
    state_test!(r#"main { br 1111 { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br T F { 2222 } } }"#, vec![BCVal::Int(5678), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br T { br T { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br T { br T { 2222 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { br F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br T { br F { 2222 } 8765 } }"#, vec![BCVal::Int(8765)]);
    state_test!(r#"main { br T { br T F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T { br T F { 2222 } 8765 } }"#, vec![BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br T { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } } }"#, vec![BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br F { 2222 } } }"#, vec![BCVal::Int(5678)]);
    state_test!(r#"main { br T { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br T F { 2222 } } }"#, vec![BCVal::Int(5678), BCVal::Bool(true)]);
    state_test!(r#"main { br T { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br F { br T { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { br T { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { br F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { br F { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { br T F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { br T F { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br T { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br T { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br F { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br T F { 2222 } } }"#, vec![]);
    state_test!(r#"main { br F { 5678 br T F { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br 1111 T { br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T { br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { br F { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 T { br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { br T F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T { br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678)]);
    state_test!(r#"main { br 1111 T { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br T F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 F { br T { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { br F { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { br T F { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br T { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br F { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br T F { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 F { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br T F { br T { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { br T { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { br F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { br F { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { br T F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { br T F { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br T F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br T F { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br T F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br T F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
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
    // Rudimentary elbr tests
    state_test!("main { br T { 9999 } elbr T { 8888 } }", vec![BCVal::Int(9999)]);
    state_test!("main { br F { 9999 } elbr T { 8888 } }", vec![BCVal::Int(8888)]);
    state_test!("main { br 7777 T { 9999 } elbr T { 8888 } }", vec![BCVal::Int(7777), BCVal::Int(9999)]);
    state_test!("main { br 7777 F { 9999 } elbr T { 8888 } }", vec![BCVal::Int(7777), BCVal::Int(8888)]);
    state_test!("main { br T { 9999 } elbr 7777 T { 8888 } }", vec![BCVal::Int(9999)]);
    state_test!("main { br F { 9999 } elbr 7777 T { 8888 } }", vec![BCVal::Int(7777), BCVal::Int(8888)]);
    // Nested statements
    state_test!("main { br T { 9999 } elbr T { br T { 7777 } elbr T { 6666 } } }", vec![BCVal::Int(9999)]);
    state_test!("main { br F { 9999 } elbr T { br T { 7777 } elbr T { 6666 } } }", vec![BCVal::Int(7777)]);
    state_test!("main { br F { 9999 } elbr T { br F { 7777 } elbr T { 6666 } } }", vec![BCVal::Int(6666)]);
}

#[test]
fn test_br_elbr_el() {
}
