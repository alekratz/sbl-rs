use errors::*;
use vm::*;
use libc::c_void;
use std::collections::HashMap;
use std::cell::RefCell;

pub struct FunState {
    pub fun: Rc<UserFun>,
    pub pc: usize,
    pub locals: HashMap<String, BCVal>,
    // TODO : callsite
}

impl FunState {
    pub fn load(&self, name: &str) -> Result<&BCVal> {
        if let Some(ref val) = self.locals.get(name) {
            Ok(val)
        } else {
            Err(
                format!("attempted to load unassigned local variable `{}`", name).into(),
            )
        }
    }

    pub fn store(&mut self, name: String, val: BCVal) {
        self.locals.insert(name.to_string(), val);
    }
}

impl From<Rc<UserFun>> for FunState {
    fn from(other: Rc<UserFun>) -> Self {
        FunState {
            fun: other,
            pc: 0,
            locals: HashMap::new(),
        }
    }
}

pub struct State {
    pub stack: Vec<BCVal>,
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

    pub fn load(&self, name: &str) -> Result<&BCVal> {
        let caller = self.current_fun();
        caller.load(name)
    }

    pub fn store(&mut self, name: String, val: BCVal) {
        let mut caller = self.current_fun_mut();
        caller.store(name, val);
    }

    pub fn peek(&self) -> Result<&BCVal> {
        if let Some(val) = self.stack.last() {
            Ok(val)
        } else {
            Err("attempted to look at the top of an empty stack".into())
        }
    }

    pub fn push(&mut self, val: BCVal) {
        self.stack.push(val)
    }

    pub fn pop(&mut self) -> Result<BCVal> {
        if let Some(val) = self.stack.pop() {
            Ok(val)
        } else {
            Err("attempted to pop an empty stack".into())
        }
    }

    pub fn popn(&mut self, n: i64) -> Result<()> {
        let len = self.stack.len();
        if n < 0 {
            return Err(
                format!(
                    "atetmpted to pop a negative number of items off of the stack (got {})",
                    n
                ).into(),
            );
        }

        let n = n as usize;
        if n > len {
            Err(
                format!(
                    "attempted to pop {} items off of a stack with only {} items",
                    n,
                    len
                ).into(),
            )
        } else {
            self.stack.truncate(len - n);
            Ok(())
        }
    }

    pub fn current_fun(&self) -> &FunState {
        self.call_stack.last().unwrap()
    }

    pub fn current_fun_mut(&mut self) -> &mut FunState {
        self.call_stack.last_mut().unwrap()
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

impl From<VM> for State {
    fn from(other: VM) -> Self {
        other.state.into_inner()
    }
}

pub struct VM {
    fun_table: FunRcTable,
    state: RefCell<State>,
    user_fun_cache: HashMap<String, Rc<UserFun>>,
}

impl VM {
    pub fn new(fun_table: BCFunTable) -> Self {
        let mut rc_table = FunRcTable::new();
        for (k, v) in fun_table {
            rc_table.insert(k, Rc::new(v));
        }
        VM {
            fun_table: rc_table,
            state: RefCell::new(State::new()),
            user_fun_cache: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Load all of the foreign functions
        for f in self.fun_table.iter().filter_map(|(_, f)| {
            if let &Fun::ForeignFun(ref f) = f as &Fun {
                Some(f)
            } else {
                None
            }
        })
        {
            f.load(&mut self.state.borrow_mut())?;
        }
        self.invoke("main")
    }

    pub fn invoke(&mut self, fun_name: &str) -> Result<()> {
        if let Some(fun) = self.user_fun_cache.get(fun_name).map(Rc::clone) {
            {
                let mut state = self.state.borrow_mut();
                state.push_fun(fun.into());
            }
            self.invoke_user_fun()?;
            {
                let mut state = self.state.borrow_mut();
                state.pop_fun();
            }
            Ok(())
        }
        else {
            let fun = self.fun_table
                .get(fun_name)
                .expect(&format!(
                    "expected function `{}` but none was found; compiler should have caught this",
                    fun_name
                ))
                .clone();
            match &fun as &Fun {
                &Fun::UserFun(ref fun) => {
                    // new user fun cache entry
                    let ptr = Rc::new(fun.clone());
                    self.user_fun_cache.insert(fun_name.to_string(), ptr.clone());
                    {
                        let mut state = self.state.borrow_mut();
                        state.push_fun(ptr.clone().into());
                    }
                    self.invoke_user_fun()?;
                    {
                        let mut state = self.state.borrow_mut();
                        state.pop_fun();
                    }
                    Ok(())
                }
                &Fun::BuiltinFun(fun) => fun(&mut self.state.borrow_mut()),
                &Fun::ForeignFun(ref fun) => fun.call(&mut self.state.borrow_mut()),
            }
        }
    }

    fn invoke_user_fun(&mut self) -> Result<()> {
        loop {
            let (bc_type, val) = {
                let state = self.state.borrow();
                let fun = state.current_fun();
                let ref bc = fun.fun.body[fun.pc];
                (bc.bc_type, bc.val.clone())
            };

            {
                match bc_type {
                    BCType::Push => {
                        let mut state = self.state.borrow_mut();
                        state.push(val.unwrap());
                        state.increment_pc();
                    }
                    BCType::PushL => {
                        let mut state = self.state.borrow_mut();
                        let item = state.pop()?;
                        let mut stack = state.pop()?;
                        if let BCVal::Stack(ref mut st) = stack {
                            st.push(item);
                        } else {
                            // This should - for now - never occur
                            unreachable!();
                        }
                        state.push(stack);
                        state.increment_pc();
                    }
                    // TODO : PushA
                    BCType::Pop => {
                        let mut state = self.state.borrow_mut();
                        let tos = state.pop()?;
                        let val = val.unwrap();
                        match val {
                            BCVal::Ident(ident) => state.store(ident, tos),
                            BCVal::Nil => { /* do nothing */ }
                            _ => unreachable!(),
                        }
                        state.increment_pc();
                    }
                    BCType::PopN => {
                        let mut state = self.state.borrow_mut();
                        let i = *val.unwrap().as_int();
                        state.popn(i)?;
                        state.increment_pc();
                    }
                    BCType::Load => {
                        let mut state = self.state.borrow_mut();
                        let val = val.unwrap();
                        let ident = val.as_ident();
                        let val = state.load(&ident)?.clone();
                        state.push(val);
                        state.increment_pc();
                    }
                    BCType::JmpZ => {
                        let mut state = self.state.borrow_mut();
                        let jump_taken = {
                            let tos = state.peek()?;
                            match tos {
                                &BCVal::Bool(false) |
                                &BCVal::Nil => true,
                                _ => false,
                            }
                        };
                        if jump_taken {
                            let addr = *val.unwrap().as_int() as usize;
                            state.set_pc(addr);
                        } else {
                            state.increment_pc();
                        }
                    }
                    BCType::Jmp => {
                        let mut state = self.state.borrow_mut();
                        let addr = *val.unwrap().as_int() as usize;
                        state.set_pc(addr);
                    }
                    BCType::Call => {
                        let val = val.unwrap();
                        let fun_name = val.as_ident();
                        self.invoke(fun_name)?;
                        let mut state = self.state.borrow_mut();
                        state.increment_pc();
                    }
                    BCType::Ret => break,
                }
            }
        }
        Ok(())
    }
}
