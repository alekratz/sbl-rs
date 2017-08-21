#[macro_use]
extern crate clap;
extern crate sbl;

use sbl::common::*;
use sbl::errors::*;
use sbl::vm::*;
use sbl::compile::*;
use sbl::internal::*;
use std::process;
use std::env;
use std::path::Path;

fn run_program<P: AsRef<Path>, Q: AsRef<Path>>(
    path: P,
    dump: bool,
    optimize: bool,
    compile_only: bool,
    search_dirs: &[Q],
) -> Result<()> {
    let filled_ast = process_source_path(path, search_dirs).chain_err(
        || "Parse error",
    )?;
    let ir_compiler = CompileIR::new(&filled_ast).builtins(&*BUILTINS);
    let compiler = CompileBytes::new(ir_compiler.compile()?);
    let fun_table = {
        let fun_table = compiler.compile().chain_err(|| "Compile error")?;
        // run optimizations
        if optimize {
            OptimizeBCInline::new(fun_table).optimize()
        } else {
            fun_table
        }
    };
    if dump {
        for f in fun_table.iter().filter_map(
            |(_, f)| if let &BCFun::UserFun(ref f) =
                f as &BCFun
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
    if !compile_only {
        let mut vm = VM::new(fun_table);
        vm.run()
    } else {
        Ok(())
    }
}

fn main() {
    let matches = clap_app!((crate_name!())=>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg DUMP: -d --dump "Dumps the bytecode of all user-defined functions")
        (@arg COMPILE_ONLY: -c --compile "Compiles only; does not run")
        (@arg OPTIMIZE: -O --optimize +takes_value
            default_value[true]
            possible_values(&["true", "false", "0", "1", "yes", "no"])
            "Whether or not to apply optimizations")
        (@arg INPUT: +required "Sets the input file to use")
        (@arg ARGV: +last ... "Any arguments to pass to the input file.")
    ).get_matches();

    let path = matches.value_of("INPUT").unwrap();

    let dump = matches.is_present("DUMP");
    let optimize = (&["true", "yes", "1"]).contains(&matches.value_of("OPTIMIZE").unwrap());
    let compile_only = matches.is_present("COMPILE_ONLY");
    let search_dirs = match env::var("SBL_PATH") {
        Ok(p) => env::split_paths(&format!(".:{}", p)).collect::<Vec<_>>(),
        _ => vec![],
    };

    if let Err(e) = run_program(path, dump, optimize, compile_only, &search_dirs) {
        print_error_chain(e);
        process::exit(1);
    }
}
