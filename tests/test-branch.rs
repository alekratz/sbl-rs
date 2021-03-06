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
    state_test!(r#"main { br T F { 5678 br F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br T { br T { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br T { 5678 br F { 2222 } } }"#, vec![BCVal::Int(5678)]);
    state_test!(r#"main { br T { br T { 2222 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 F { br T { 2222 } } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 T { 5678 br { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 { br T F { 2222 } 8765 } }"#, vec![BCVal::Bool(true), BCVal::Int(8765)]);
    state_test!(r#"main { br F { br T { 2222 } } }"#, vec![]);
    state_test!(r#"main { br 1111 T F { br T F { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br T { br T F { 2222 } } }"#, vec![BCVal::Bool(true)]);
    state_test!(r#"main { br 1111 T F { 5678 br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
    state_test!(r#"main { br F { 5678 br T { 2222 } 8765 } }"#, vec![]);
    state_test!(r#"main { br 1111 { 5678 br { 2222 } } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 F { 5678 br T F { 2222 } 8765 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br 1111 T { br T { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T F { br T { 2222 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true)]);
}

#[test]
fn test_br_el() {
    state_test!(r#"main { br 1111 F { 5678 br T { 2222 } el { 3333 } 8765 } el { 5678 br T { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br F { 2222 } el { 3333 } 8765 } el { 5678 br F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br T { br T F { 2222 } el { 3333 } } el { br T F { 2222 } el { 3333 } } }"#, vec![BCVal::Bool(true), BCVal::Int(3333)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } el { 3333 } } el { 5678 br T { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 F { 5678 br F { 2222 } el { 3333 } } el { 5678 br F { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(3333)]);
    state_test!(r#"main { br F { br T F { 2222 } el { 3333 } 8765 } el { br T F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Bool(true), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br F { 2222 } el { 3333 } 8765 } el { 5678 br F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br F { 2222 } el { 3333 } } el { 5678 br F { 2222 } el { 3333 } } }"#, vec![BCVal::Int(5678), BCVal::Int(3333)]);
    state_test!(r#"main { br T F { 5678 br { 2222 } el { 3333 } } el { 5678 br { 2222 } el { 3333 } } }"#, vec![BCVal::Bool(true), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 F { 5678 br T F { 2222 } el { 3333 } 8765 } el { 5678 br T F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Bool(true), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { br T F { 2222 } el { 3333 } 8765 } el { br T F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br T { br F { 2222 } el { 3333 } 8765 } el { br F { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br F { br T { 2222 } el { 3333 } 8765 } el { br T { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } el { 3333 } 8765 } el { 5678 br T { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T F { br T F { 2222 } el { 3333 } } el { br T F { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Bool(true), BCVal::Int(3333)]);
    state_test!(r#"main { br 1111 T F { 5678 br T { 2222 } el { 3333 } } el { 5678 br T { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T F { 5678 br { 2222 } el { 3333 } } el { 5678 br { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 { br T { 2222 } el { 3333 } 8765 } el { br T { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 F { 5678 br T { 2222 } el { 3333 } } el { 5678 br T { 2222 } el { 3333 } } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T { br T { 2222 } el { 3333 } 8765 } el { br T { 2222 } el { 3333 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(2222), BCVal::Int(8765)]);
}

#[test]
fn test_br_elbr() {
    state_test!(r#"main { br T { 5678 br T { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } elbr T { 4444 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } elbr T { 4444 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 F { 5678 br F { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(4444)]);
    state_test!(r#"main { br T { br T { 2222 } elbr T { 4444 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br T { br T { 2222 } elbr T { 4444 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(2222)]);
    state_test!(r#"main { br 1111 T { br F { 2222 } elbr F { 4444 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111)]);
    state_test!(r#"main { br T F { br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br T F { br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br F { br F { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br F { 5678 br T { 2222 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br F { 5678 br T { 2222 } } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 T { br F { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br F { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(4444), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T { 5678 br T { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br T F { 5678 br T { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 F { 5678 br T { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } }"#, vec![BCVal::Int(1111), BCVal::Int(4444)]);
}

#[test]
fn test_br_elbr_el() {
    state_test!(r#"main { br F { br F { 2222 } el { 3333 } } elbr T { 4444 } elbr F { 4444 } el { br F { 2222 } 8765 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br T { 5678 br F { 2222 } el { 3333 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(3333), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br F { 2222 } elbr T { 4444 } elbr F { 4444 } el { 3333 } } elbr T { 4444 } elbr F { 4444 } el { 5678 br F { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 T F { 5678 br F { 2222 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br F { 2222 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 T { br F { 2222 } elbr F { 4444 } el { 3333 } } elbr T { 4444 } elbr F { 4444 } el { br F { 2222 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(3333)]);
    state_test!(r#"main { br F { br T { 2222 } elbr T { 4444 } elbr F { 4444 } el { 3333 } } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br F { br T { 2222 } elbr T { 4444 } elbr F { 4444 } el { 3333 } } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br T { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { br T { 2222 } elbr T { 4444 } el { 3333 } 8765 } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr T { 4444 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { br T { 2222 } elbr T { 4444 } el { 3333 } 8765 } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr T { 4444 } 8765 } }"#, vec![BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 T F { br T { 2222 } elbr F { 4444 } } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr F { 4444 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 T F { br T { 2222 } elbr F { 4444 } } elbr T { 4444 } elbr F { 4444 } el { br T { 2222 } elbr F { 4444 } } }"#, vec![BCVal::Int(1111), BCVal::Bool(true), BCVal::Int(4444)]);
    state_test!(r#"main { br F { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br F { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 F { br F { 2222 } elbr T { 4444 } 8765 } elbr T { 4444 } elbr F { 4444 } el { br F { 2222 } elbr T { 4444 } 8765 } }"#, vec![BCVal::Int(1111), BCVal::Int(4444)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } el { 3333 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
    state_test!(r#"main { br 1111 { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } el { 3333 } 8765 } elbr T { 4444 } elbr F { 4444 } el { 5678 br T { 2222 } elbr T { 4444 } elbr F { 4444 } 8765 } }"#, vec![BCVal::Int(5678), BCVal::Int(2222), BCVal::Int(8765)]);
}
