use errors::*;
use vm::*;
use std::collections::HashMap;
use std::cmp::Ordering;

pub type BuiltinFun = fn(&mut State) -> Result<()>;

lazy_static! {
    pub static ref BUILTINS: HashMap<&'static str, BuiltinFun> = {
        hashmap! {
            // Operations
            "+" => plus as BuiltinFun,  // for some reason this cascades down the list
            "-" => minus,
            "*" => times,
            "/" => divide,
            "|" => bit_or,
            "==" => equals,
            "!=" => not_equals,
            "<" => less_than,
            ">" => greater_than,
            "<=" => lt_equals,
            ">=" => gt_equals,

            // Stack functions
            "^" => tos,
            "$" => stack_size,

            // Local stack functions
            "^push" => push,
            "^pop" => pop,
            "^len" => len_o,
            "!len" => len_c,

            // Quality of life functions
            "^print" => print_o,
            "!print" => print_c,
            "^println" => println_o,
            "!println" => println_c,

            // Debug functions
            "^dump_stack" => dump_stack,
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
        Val::Int(i1) => {
            if let Val::Int(i2) = rhs {
                state.push(Val::Int(i1 + i2));
            }
        }
        _ => return Err("Addition between non-integers is not allowed".into()),
    }
    Ok(())
}

fn minus(state: &mut State) -> Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => {
            if let Val::Int(i2) = rhs {
                state.push(Val::Int(i1 - i2));
            }
        }
        _ => return Err("Subtraction between non-integers is not allowed".into()),
    }
    Ok(())
}

fn times(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => {
            if let Val::Int(i2) = rhs {
                state.push(Val::Int(i1 * i2));
            }
        }
        _ => return Err("Multiplication between non-integers is not allowed".into()),
    }
    Ok(())
}

fn divide(state: &mut State) -> Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;
    // TODO : addition between different types
    match lhs {
        Val::Int(i1) => {
            if let Val::Int(i2) = rhs {
                state.push(Val::Int(i1 / i2));
            }
        }
        _ => return Err("Division between non-integers is not allowed".into()),
    }
    Ok(())
}

fn bit_or(state: &mut State) -> Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;
    match lhs {
        Val::Int(i1) => {
            if let Val::Int(i2) = rhs {
                state.push(Val::Int(i1 | i2));
            }
        }
        _ => return Err("Bitwise-or between non-integers is not allowed".into()),
    }
    Ok(())
}

fn equals(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    state.push(Val::Bool(lhs == rhs));
    Ok(())
}

fn not_equals(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    state.push(Val::Bool(lhs != rhs));
    Ok(())
}

fn less_than(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    state.push(Val::Bool(lhs.compare(&rhs)? == Ordering::Less));
    Ok(())
}

fn greater_than(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    state.push(Val::Bool(lhs.compare(&rhs)? == Ordering::Greater));
    Ok(())
}

fn lt_equals(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    let cmp = lhs.compare(&rhs)?;
    state.push(Val::Bool(cmp == Ordering::Less || cmp == Ordering::Equal));
    Ok(())
}

fn gt_equals(state: &mut State) -> Result<()> {
    let lhs = state.pop()?;
    let rhs = state.pop()?;
    let cmp = lhs.compare(&rhs)?;
    state.push(Val::Bool(
        cmp == Ordering::Greater || cmp == Ordering::Equal,
    ));
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
 * Local stack functions
 */

fn push(state: &mut State) -> Result<()> {
    let tos = state.pop()?;
    let mut stack = state.pop()?;
    if let Val::Stack(ref mut st) = stack {
        st.push(tos);
    } else {
        return Err(
            format!(
                "expected TOS item to be stack; instead got {}",
                stack.type_string()
            ).into(),
        );
    }

    state.push(stack);
    Ok(())
}

fn pop(state: &mut State) -> Result<()> {
    let mut stack = state.pop()?;
    let popped: Val = if let Val::Stack(ref mut st) = stack {
        if st.len() > 0 {
            Ok(st.pop().unwrap()) as Result<Val>
        } else {
            Err("attempted to pop empty TOS item".into()) as Result<Val>
        }
    } else {
        Err(
            format!(
                "expected TOS item to be stack; instead got {}",
                stack.type_string()
            ).into(),
        ) as Result<Val>
    }?;

    state.push(stack);
    state.push(popped);
    Ok(())
}

fn len_o(state: &mut State) -> Result<()> {
    let len = {
        let p = state.peek()?;
        if p.is_stack() {
            p.stack().len()
        } else if p.is_string() {
            p.string().len()
        } else {
            return Err(
                format!(
                    "expected TOS item to be stack or string; instead got {}",
                    p.type_string()
                ).into(),
            );
        }
    };
    state.push(Val::Int(len as i64));
    Ok(())
}

fn len_c(state: &mut State) -> Result<()> {
    let len = {
        let p = state.pop()?;
        if p.is_stack() {
            p.stack().len()
        } else if p.is_string() {
            p.string().len()
        } else {
            return Err(
                format!(
                    "expected TOS item to be stack or string; instead got {}",
                    p.type_string()
                ).into(),
            );
        }
    };
    state.push(Val::Int(len as i64));
    Ok(())
}

/*
 * QOL functions
 */

fn print_o(state: &mut State) -> Result<()> {
    let tos = state.peek()?;
    print!("{}", tos);
    Ok(())
}

fn print_c(state: &mut State) -> Result<()> {
    print_o(state)?;
    state.pop()?;
    Ok(())
}

fn println_o(state: &mut State) -> Result<()> {
    let tos = state.peek()?;
    println!("{}", tos);
    Ok(())
}

fn println_c(state: &mut State) -> Result<()> {
    println_o(state)?;
    state.pop()?;
    Ok(())
}

fn dump_stack(state: &mut State) -> Result<()> {
    eprintln!("- dumping global stack -------------------------------------------------");
    let mut c = 0;
    for f in state.stack.iter().rev() {
        if c == 0 {
            eprintln!("   top: {:?}", f);
        } else if c == state.stack.len() - 1 {
            eprintln!("bottom: {:?}", f);
        } else {
            eprintln!("{:>6}: {:?}", c, f);
        }
        c += 1;
    }
    eprintln!("{}", "-".repeat(72));
    Ok(())
}
