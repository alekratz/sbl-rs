use errors::*;
use vm::*;
use std::collections::HashMap;

type BuiltinFn = fn(&mut State) -> Result<()>;

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, BuiltinFn> = {
        hashmap! {
            "+" => plus as BuiltinFn,  // for some reason this cascades down the list
            "-" => minus,
            "*" => times,
            "/" => divide,
        }
    };
}

fn plus(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    // TODO : addition between different types
    Ok(())
}

fn minus(state: &mut State) -> Result<()> {
    Ok(())
}

fn times(state: &mut State) -> Result<()> {
    Ok(())
}

fn divide(state: &mut State) -> Result<()> {
    Ok(())
}
