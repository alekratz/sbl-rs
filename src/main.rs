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
extern crate libc;
extern crate libffi;

mod syntax;
mod compile;
mod vm;
#[macro_use]
mod common;

mod errors {
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

use common::*;
use errors::*;
use vm::*;
use compile::*;
use std::process;
use std::env;
use std::path::Path;

fn run_program<P: AsRef<Path>, Q: AsRef<Path>>(
    path: P,
    dump: bool,
    search_dirs: &[Q],
) -> Result<()> {
    let filled_ast = process_source_path(path, search_dirs).chain_err(
        || "Parse error",
    )?;
    let compiler = CompileBytes::new(filled_ast)
        .builtins(&*BUILTINS);
    let fun_table = compiler.compile().chain_err(|| "Compile error")?;
    let fun_table = OptimizeInline::new(fun_table)
        .optimize();
    if dump {
        for f in fun_table.iter().filter_map(
            |(_, f)| if let &Fun::UserFun(ref f) =
                f as &Fun
            {
                Some(f)
            } else {
                None
            },
        )
        {
            eprintln!("- {} {}", &f.name, "-".repeat(69 - f.name.len()));
            f.dump();
        }
    }
    let mut vm = VM::new(fun_table);
    vm.run()
}

fn main() {
    let matches = clap_app!((crate_name!())=>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg DUMP: -d --dump "Dumps the bytecode of all user-defined functions")
        (@arg INPUT: +required "Sets the input file to use")
        (@arg ARGV: +last ... "Any arguments to pass to the input file.")
    ).get_matches();

    let path = matches.value_of("INPUT").unwrap();

    let dump = matches.is_present("DUMP");

    let search_dirs = match env::var("SBL_PATH") {
        Ok(p) => env::split_paths(&format!(".:{}", p)).collect::<Vec<_>>(),
        _ => vec![],
    };

    if let Err(e) = run_program(path, dump, &search_dirs) {
        print_error_chain(e);
        process::exit(1);
    }
}
