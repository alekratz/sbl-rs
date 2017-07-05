#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate clap;

mod syntax;
mod vm;
#[macro_use]
mod common;
mod errors { error_chain!{} }

use common::read_file;
use syntax::*;
use std::process;

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

    // set up tokenizer and parser
    let tokenizer = Tokenizer::new(input_file, &contents);
    let mut parser = Parser::new(tokenizer);
    println!("{:#?}", parser.parse());
}
