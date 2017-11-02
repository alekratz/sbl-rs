use prelude::*;
use libc::c_void;
use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct BCFunState {
    pub fun: Rc<BCUserFun>,
    pub pc: usize,
    pub locals: Vec<Option<BCVal>>,
    // TODO : callsite
}

impl BCFunState {
    pub fn load(&self, varnum: usize) -> Result<&BCVal> {
        if let Some(ref val) = self.locals[varnum] {
            Ok(val)
        } else {
            let ref name = self.fun.locals[varnum];
            Err(
                format!("attempted to load unassigned local variable `{}`", name).into(),
            )
        }
    }

    pub fn store(&mut self, varnum: usize, val: BCVal) {
        self.locals[varnum] = Some(val);
    }
}

impl From<Rc<BCUserFun>> for BCFunState {
    fn from(other: Rc<BCUserFun>) -> Self {
        BCFunState {
            fun: other.clone(),
            pc: 0,
            locals: vec!(None; other.locals.len()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub stack: Vec<BCVal>,
    pub call_stack: Vec<BCFunState>,
    pub dl_handles: BTreeMap<String, *mut c_void>,
    pub foreign_functions: BTreeMap<String, *mut c_void>,
}

impl State {
    pub fn new() -> Self {
        State {
            stack: vec![],
            call_stack: vec![],
            dl_handles: BTreeMap::new(),
            foreign_functions: BTreeMap::new(),
        }
    }

    /// Dumps the current state to stderr
    pub fn dump(&self) {
        eprintln!("Call stack (address fname)");
        for fun in &self.call_stack {
            eprintln!("{:06} {}", fun.pc, fun.fun.name());
        }
        eprintln!();
        eprintln!("Stack (top to bottom)");
        for val in &self.stack {
            eprintln!("    {}", val);
        }
    }

    pub fn load(&self, varnum: usize) -> Result<&BCVal> {
        let caller = self.current_fun();
        caller.load(varnum)
    }

    pub fn store(&mut self, varnum: usize, val: BCVal) {
        let caller = self.current_fun_mut();
        caller.store(varnum, val);
    }

    pub fn peek(&self) -> Result<&BCVal> {
        if let Some(val) = self.stack.last() {
            Ok(val)
        } else {
            Err("attempted to look at the top of an empty stack".into())
        }
    }

    pub fn push_all(&mut self, vals: &[BCVal]) {
        self.stack.extend_from_slice(vals);
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

    pub fn current_fun(&self) -> &BCFunState {
        self.call_stack.last().unwrap()
    }

    pub fn current_fun_mut(&mut self) -> &mut BCFunState {
        self.call_stack.last_mut().unwrap()
    }

    pub fn push_fun(&mut self, fun_state: BCFunState) {
        self.call_stack.push(fun_state);
    }

    pub fn pop_fun(&mut self) -> BCFunState {
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

#[derive(Clone)]
pub struct VM {
    fun_table: BCFunRcTable,
    state: RefCell<State>,
    user_fun_cache: BTreeMap<String, Rc<BCUserFun>>,
}

impl VM {
    pub fn new(fun_table: BCFunTable) -> Self {
        let mut rc_table = BCFunRcTable::new();
        // TODO : itertools RC iterator
        for (k, v) in fun_table {
            rc_table.insert(k, Rc::new(v));
        }
        VM {
            fun_table: rc_table,
            state: RefCell::new(State::new()),
            user_fun_cache: BTreeMap::new(),
        }
    }

    pub fn add_fun(&mut self, name: String, fun: BCFun) {
        // XXX - I don't like this function, is there a better way we can update a funtable owned
        // by a VM? (probably not)
        self.fun_table.insert(name, Rc::new(fun));
    }

    pub fn run(&mut self) -> Result<()> {
        // Load all of the foreign functions
        for f in self.fun_table.iter().filter_map(|(_, f)| {
            if let &Fun::ForeignFun(ref f) = f as &BCFun {
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
            match &fun as &BCFun {
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

    pub fn inject_user_fun(&mut self, fun: BCUserFun) -> Result<()> {
        let rc = Rc::new(fun);
        {
            let mut state = self.state.borrow_mut();
            state.push_fun(rc.into());
        }
        self.invoke_user_fun()?;
        {
            let mut state = self.state.borrow_mut();
            state.pop_fun();
        }
        Ok(())
    }

    /// Prints out the VM state to the command line.
    /// Useful for crash reports.
    pub fn dump_state(&self) {
        let state = self.state.borrow();
        state.dump();
    }

    fn invoke_user_fun(&mut self) -> Result<()> {
        loop {
            let (bc_type, target, val, fun) = {
                let state = self.state.borrow();
                let fun = state.current_fun();
                let ref bc = fun.fun.body[fun.pc];
                (bc.bc_type, bc.target.clone(), bc.val.clone(), fun.fun.clone())
            };

            {
                match bc_type {
                    BCType::Push => {
                        let mut state = self.state.borrow_mut();
                        state.push_all(val.unwrap().as_push_all());
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
                    BCType::Pop => {
                        // TODO : change POP to use 'target' instead of 'val'
                        // TODO : change POP to use 'target' instead of 'val'
                        // TODO : change POP to use 'target' instead of 'val'
                        // TODO : change POP to use 'target' instead of 'val'
                        // TODO : change POP to use 'target' instead of 'val'
                        let mut state = self.state.borrow_mut();
                        let tos = state.pop()?;
                        let varnum = *val.unwrap()
                            .as_address();
                        state.store(varnum, tos);
                        state.increment_pc();
                    }
                    BCType::PopN => {
                        let mut state = self.state.borrow_mut();
                        let i = *val.unwrap().as_int();
                        state.popn(i)?;
                        state.increment_pc();
                    }
                    BCType::PopDiscard => {
                        let mut state = self.state.borrow_mut();
                        state.pop()?;
                        state.increment_pc();
                    }
                    BCType::Load => {
                        let mut state = self.state.borrow_mut();
                        let varnum = *val.unwrap()
                            .as_address();
                        let val = state.load(varnum)?.clone();
                        state.push(val);
                        state.increment_pc();
                    }
                    BCType::Store => {
                        let mut state = self.state.borrow_mut();
                        let varnum = *target.unwrap()
                            .as_address();
                        let val = val.unwrap();
                        state.store(varnum, val);
                        state.increment_pc();
                    }
                    BCType::Jmp => {
                        let mut state = self.state.borrow_mut();
                        let addr = *val.unwrap().as_address();
                        state.set_pc(addr);
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
                            let addr = *val.unwrap().as_address();
                            state.set_pc(addr);
                        } else {
                            state.increment_pc();
                        }
                    }
                    BCType::SymJmp => {
                        let mut state = self.state.borrow_mut();
                        let symbol = *val.unwrap().as_int();
                        let addr = fun.get_label_address(symbol);
                        state.set_pc(addr);
                    }
                    BCType::SymJmpZ => {
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
                            let symbol = *val.unwrap().as_int();
                            let addr = fun.get_label_address(symbol);
                            state.set_pc(addr);
                        } else {
                            state.increment_pc();
                        }
                    }
                    BCType::Call => {
                        let val = val.unwrap();
                        let fun_name = val.as_ident();
                        self.invoke(fun_name)?;
                        let mut state = self.state.borrow_mut();
                        state.increment_pc();
                    }
                    BCType::Ret => break,
                    BCType::Nop | BCType::Label => {
                        let mut state = self.state.borrow_mut();
                        state.increment_pc();
                    },
                }
            }
        }
        Ok(())
    }
}
