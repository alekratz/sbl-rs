# SBL - Stack Based Language

[![Build Status](https://travis-ci.org/alekratz/sbl-rs.svg?branch=master)](https://travis-ci.org/alekratz/sbl-rs)

Original name, right?

If you're just getting started, check the [wiki](https://github.com/alekratz/sbl-rs/wiki)!

## This is what it looks like:
```
# Calculates n factorial (n!).
fact {
    # duplicate and compare to zero
    ^ 0 ==
    br {
        .@
        # pop off to nothing, and push a 1
        .@ 1
    }
    el {
        .@
        .x       # pop into x
        x 1 -    # push a copy and subtract 1 from it
        fact     # call factorial
        x *      # multiply whatever our factorial is by x
    }
}

main {
    @ 5 4 3 2 1
    loop { fact println }
}
```

# Installing
This project requires Rust 1.19 stable (nightly and beta should also work).

```commandline
git clone https://github.com/alekratz/sbl-rs
cd sbl
cargo build
```

If you want to build a release version, tack `--release` to the end of the
`cargo` invocation.

Optionally if you want to run `sbl-rs` from the command line, run
`cargo install`. Otherwise, you can run the program from
`target/{debug,release}/sbl-rs file.sbl` or `cargo run -- file.sbl`.

# Basic usage
All SBL supports right now is running directly from a file. If you wish to import code from multiple
files, multiple files may be supplied from the command line.

## Examples
* `sbl-rs test.sbl`

Note that SBL files must not contain duplicate functions; this is a compile-time error if they do.

# Grammar
You can check out the grammar in [GRAMMAR.md](GRAMMAR.md).

# Features
* Terse syntax
* Branches
* Loops
* Recursive functions
* Order-agnostic function definition
* Simple, LL(0) grammar (not regular, but close)
* Built-in function support
* A handful of primitive types
* File path imports
    * Include paths, too!
* Ability to call (some) foreign functions
* More to come...

# Non-features
Or, "room for improvement"

* Lightning-fast virtual machine and compiler implemented in Python
* No savable bytecode (see [#10](https://github.com/alekratz/sbl-rs/issues/10))
* No base or standard library (see [#11](https://github.com/alekratz/sbl-rs/issues/11))

# Planned features
TODO

# Releases
None yet!

# Contributing
Contributions are welcome and happily accepted. Check out [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

# License
Apache2. See LICENSE file for details.
