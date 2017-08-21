#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate enum_methods;
extern crate libc;
extern crate libffi;
extern crate itertools;
extern crate petgraph;

pub mod syntax;
pub mod compile;
pub mod vm;
pub mod common;
pub mod ir;
pub mod bc;

pub mod errors {
    use common::*;

    error_chain! {
        errors {
            Ranged(range: Range) {
                description("Ranged error")
                display("in {}", range)
            }
        }
    }

    impl From<Range> for ErrorKind {
        fn from(r: Range) -> ErrorKind {
            ErrorKind::Ranged(r)
        }
    }
}
