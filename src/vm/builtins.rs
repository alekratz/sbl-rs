use errors::*;
use vm::*;
use std::collections::HashMap;

type BuiltinFn = fn(&mut State) -> Result<()>;

lazy_static! {
    pub(in vm) static ref BUILTINS: HashMap<&'static str, BuiltinFn> = {
        hashmap! {
            "+" => plus as BuiltinFn,  // for some reason this cascades down the list
            "-" => minus,
            "*" => times,
            "/" => divide,
            "==" => equals,

            "^" => tos,
            "$" => stack_size,

            "print" => print,
            "println" => println,
        }
    };
}

/*
 * Operations
 */

fn plus(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => if let Val::Int(i2) = rhs { state.push(Val::Int(i1 + i2)); },
        _ => return Err("Addition between non-integers is not allowed".into()),
    }
    Ok(())
}

fn minus(state: &mut State) -> Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => if let Val::Int(i2) = rhs { state.push(Val::Int(i1 - i2)); },
        _ => return Err("Subtraction between non-integers is not allowed".into()),
    }
    Ok(())
}

fn times(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => if let Val::Int(i2) = rhs { state.push(Val::Int(i1 * i2)); },
        _ => return Err("Multiplication between non-integers is not allowed".into()),
    }
    Ok(())
}

fn divide(state: &mut State) -> Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => if let Val::Int(i2) = rhs { state.push(Val::Int(i1 / i2)); },
        _ => return Err("Division between non-integers is not allowed".into()),
    }
    Ok(())
}

fn equals(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    state.push(Val::Bool(lhs == rhs));
    Ok(())
}

/*
 * Stack access functions
 */
fn tos(state: &mut State) -> Result<()> {
    let tos = state.peek()?.clone();
    state.push(tos);
    Ok(())
}

fn stack_size(state: &mut State) -> Result<()> {
    let size = Val::Int(state.stack_size() as i64);
    state.push(size);
    Ok(())
}

/*
 * QOL functions
 */

fn print(state: &mut State) -> Result<()> {
    let tos = state.pop()?;
    print!("{}", tos);
    Ok(())
}

fn println(state: &mut State) -> Result<()> {
    let tos = state.pop()?;
    println!("{}", tos);
    Ok(())
}
