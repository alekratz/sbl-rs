#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate enum_methods;

mod syntax;
mod vm;
#[macro_use]
mod common;
mod errors { error_chain!{} }

use common::read_file;
use errors::*;
use syntax::*;
use vm::*;
use error_chain::ChainedError;
use std::process;

fn process_contents(path: &str, contents: String) -> Result<()> {
    // set up tokenizer and parser
    let tokenizer = Tokenizer::new(path, &contents);
    let mut parser = Parser::new(tokenizer);
    let ast = parser.parse()?;
    let compiler = Compiler::new(&ast);
    let fun_table = compiler.compile()?;
    let mut vm = VM::new(fun_table);
    vm.run()
}

fn print_error_chain<T: ChainedError>(err_chain: T) {
    printerr!("{}", err_chain.iter().nth(0).unwrap());
    for err in err_chain.iter().skip(1) {
        printerr!("... {}", err);
    }
}

fn main() {
    let matches = clap_app!((crate_name!())=>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg INPUT: +required "Sets the input file to use")
        (@arg ARGV: +last ... "Any arguments to pass to the input file.")
    ).get_matches();

    let input_file = matches.value_of("INPUT")
        .unwrap();

    let contents = match read_file(input_file) {
        Ok(c) => c,
        Err(e) => {
            printerr!("error reading `{}`: {}", input_file, e);
            process::exit(1);
        }
    };

    if let Err(e) = process_contents(input_file, contents) {
        print_error_chain(e);
        process::exit(1);
    }
}
