use errors::*;
use vm::*;
use vm::builtins::*;
use libc::c_void;
use std::collections::HashMap;
use std::cell::RefCell;

pub struct FunState {
    pub fun: Rc<Fun>,
    pub pc: usize,
    pub locals: HashMap<String, Val>,
    // TODO : callsite
}

impl FunState {
    pub fn load(&self, name: &str) -> Result<&Val> {
        if let Some(ref val) = self.locals.get(name) {
            Ok(val)
        }
        else {
            Err(format!("attempted to load unassigned local variable `{}`", name).into())
        }
    }

    pub fn store(&mut self, name: String, val: Val) {
        self.locals.insert(name.to_string(), val);
    }
}

impl From<Rc<Fun>> for FunState {
    fn from(other: Rc<Fun>) -> Self {
        FunState {
            fun: other,
            pc: 0,
            locals: HashMap::new(),
        }
    }
}

pub(in vm) struct State {
    pub stack: Vec<Val>,
    pub call_stack: Vec<FunState>,
    pub dl_handles: HashMap<String, *mut c_void>,
    pub foreign_functions: HashMap<String, *mut c_void>,
}

impl State {
    pub fn new() -> Self {
        State {
            stack: vec![],
            call_stack: vec![],
            dl_handles: HashMap::new(),
            foreign_functions: HashMap::new(),
        }
    }

    pub fn load(&self, name: &str) -> Result<&Val> {
        let caller = self.current_fun();
        caller.load(name)
    }

    pub fn store(&mut self, name: String, val: Val) {
        let mut caller = self.current_fun_mut();
        caller.store(name, val);
    }

    pub fn peek(&self) -> Result<&Val> {
        if let Some(val) = self.stack.last() {
            Ok(val)
        }
        else {
            Err("attempted to look at the top of an empty stack".into())
        }
    }

    pub fn push(&mut self, val: Val) {
        self.stack.push(val)
    }

    pub fn pop(&mut self) -> Result<Val> {
        if let Some(val) = self.stack.pop() {
            Ok(val)
        }
        else {
            Err("attempted to pop an empty stack".into())
        }
    }

    pub fn popn(&mut self, n: i64) -> Result<()> {
        let len = self.stack.len();
        if n < 0 {
            return Err(format!("atetmpted to pop a negative number of items off of the stack (got {})", n).into());
        }

        let n = n as usize;
        if n > len {
            Err(format!("attempted to pop {} items off of a stack with only {} items", n, len).into())
        }
        else {
            self.stack.truncate(len - n);
            Ok(())
        }
    }

    pub fn current_fun(&self) -> &FunState {
        self.call_stack.last()
            .unwrap()
    }

    pub fn current_fun_mut(&mut self) -> &mut FunState {
        self.call_stack.last_mut()
            .unwrap()
    }

    pub fn push_fun(&mut self, fun_state: FunState) {
        self.call_stack.push(fun_state);
    }

    pub fn pop_fun(&mut self) -> FunState {
        self.call_stack.pop().unwrap()
    }

    pub fn increment_pc(&mut self) {
        let fun = self.current_fun_mut();
        fun.pc += 1;
    }

    pub fn set_pc(&mut self, pc: usize) {
        let fun = self.current_fun_mut();
        fun.pc = pc;
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }
}

pub struct VM {
    fun_table: FunTable,
    foreign_funs: Vec<ForeignFn>,
    state: RefCell<State>,
}

impl VM {
    pub fn new(fun_table: FunTable) -> Self {
        VM {
            fun_table,
            foreign_funs: Vec::new(),  // TODO : Fill this in
            state: RefCell::new(State::new()),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        for ref f in self.foreign_funs {
            f.load(&mut self.state.borrow_mut());
        }
        self.invoke("main")
    }

    fn invoke(&mut self, fun_name: &str) -> Result<()> {
        if let Some(fun) = self.fun_table.get(fun_name) {
            let mut state = self.state
                .borrow_mut();
            state.push_fun(fun.clone().into());
        }
        else if let Some(fun) = BUILTINS.get(fun_name) {
            return fun(&mut self.state.borrow_mut());
        }
        else {
            return Err(format!("tried to call undefined function `{}`", fun_name).into());
        }

        loop {
            let (bc_type, val) = {
                let state = self.state.borrow();
                let fun = state.current_fun();
                let ref bc = fun.fun.body()[fun.pc];
                (*bc.bc_type(), bc.val_clone())
            };

            {
                match bc_type {
                    BcType::Push => {
                        let mut state = self.state.borrow_mut();
                        state.push(val.unwrap());
                        state.increment_pc();
                    },
                    BcType::PushL => {
                        let mut state = self.state.borrow_mut();
                        let item = state.pop()?;
                        let mut stack = state.pop()?;
                        if let Val::Stack(ref mut st) = stack {
                            st.push(item);
                        }
                        else {
                            // This should - for now - never occur
                            unreachable!();
                        }
                        state.push(stack);
                        state.increment_pc();
                    },
                    BcType::Pop => {
                        let mut state = self.state.borrow_mut();
                        let tos = state.pop()?;
                        let val = val.unwrap();
                        match val {
                            Val::Ident(ident) => state.store(ident, tos),
                            Val::Nil => { /* do nothing */ },
                            _ => unreachable!(),
                        }
                        state.increment_pc();
                    },
                    BcType::PopN => {
                        let mut state = self.state.borrow_mut();
                        let i = val.unwrap().int();
                        state.popn(i)?;
                        state.increment_pc();
                    },
                    BcType::Load => {
                        let mut state = self.state.borrow_mut();
                        let val = val.unwrap();
                        let ident = val.ident();
                        let val = state.load(&ident)?
                            .clone();
                        state.push(val);
                        state.increment_pc();
                    },
                    BcType::JmpZ => {
                        let mut state = self.state.borrow_mut();
                        let jump_taken = {
                            let tos = state.peek()?;
                            match tos {
                                &Val::Bool(false) | &Val::Nil => true,
                                _ => false,
                            }
                        };
                        if jump_taken {
                            let addr = val.unwrap().int() as usize;
                            state.set_pc(addr);
                        }
                        else {
                            state.increment_pc();
                        }
                    },
                    BcType::Jmp => {
                        let mut state = self.state.borrow_mut();
                        let addr = val.unwrap().int() as usize;
                        state.set_pc(addr);
                    },
                    BcType::Call => {
                        let val = val.unwrap();
                        let fun_name = val.ident();
                        self.invoke(fun_name)?;
                        let mut state = self.state.borrow_mut();
                        state.increment_pc();
                    },
                    BcType::Ret => break,
                }
            }
        }

        {
            let mut state = self.state.borrow_mut();
            state.pop_fun();
        }
        Ok(())
    }
}
