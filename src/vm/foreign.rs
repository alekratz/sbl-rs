use errors::*;
use syntax::{ForeignFn, ItemType};
use vm::{State, Val};
use libc::{self, RTLD_NOW, c_char};
use libffi::low::CodePtr;
use libffi::high::call::{call, Arg};
use std::ffi::CString;

#[derive(PartialEq, Clone, Debug)]
enum FfiVal {
    Int(i64),
    Char(u8),
    String(*const c_char),
    Bool(i32),
    //Stack(&'a *const c_void),
    Nil(()),
}

impl<'a> FfiVal {
    pub fn to_arg(&'a self) -> Arg<'a> {
        match self {
            &FfiVal::Int(ref i) => Arg::new(i),
            &FfiVal::Char(ref c) => Arg::new(c),
            &FfiVal::String(ref s) => Arg::new(s),
            &FfiVal::Bool(ref i) => Arg::new(i),
            &FfiVal::Nil(ref t) => Arg::new(t),
        }
    }
}

fn ff_key(lib: &str, name: &str) -> String {
    format!("{}_{}", lib, name)
}

impl ForeignFn {
    /// Makes a call into a foreign function.
    pub(in vm) fn call(&self, state: &mut State) -> Result<()> {
        // string pool holds all of the strings that we have to re-allocate as CStrings
        let mut string_pool = vec![];
        let mut val_args = vec![];
        eprintln!("calling {}", &self.name);
        for p in &self.params {
            let arg = state.pop()?;
            let matches = match *p {
                ItemType::Int(_) => arg.is_int(),
                ItemType::Char(_) => arg.is_char(),
                ItemType::String(_) => arg.is_string(),
                ItemType::Bool(_) => arg.is_bool(),
                //ItemType::Stack(_) => arg.is_stack(),
                ItemType::Nil => arg.is_nil(),
                _ => unreachable!(),
            };
            if !matches {
                return Err(
                    format!(
                        "expected argument of type {}; instead got {}",
                        p.type_string(),
                        arg.type_string()
                    ).into(),
                );
            }
            eprintln!("arg: {:?}", arg);
            val_args.push(arg);
        }

        // convert the values into FFI arguments, which own copies of their appropriate types
        // NOTE: ffi_args needs to live while val_args does, because of string pointers. I'm not
        // sure how string pointers don't need a lifetime, but I'm not going to take risks.
        let ffi_args = val_args
            .iter()
            .map(|v| match v {
                &Val::Int(i) => FfiVal::Int(i),
                &Val::Char(c) => FfiVal::Char(c as u8),
                &Val::String(ref s) => {
                    let c_str = CString::new(s.as_str()).unwrap();
                    string_pool.push(c_str);
                    FfiVal::String(string_pool.last().as_ref().unwrap().as_ptr())
                }
                &Val::Bool(b) => FfiVal::Bool(if b { 1 } else { 0 }),
                &Val::Nil => FfiVal::Nil(()),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let args = ffi_args.iter().map(|ref v| v.to_arg()).collect::<Vec<_>>();

        let result = {
            // doing the function gathering second (rather than first) because we need an immutable
            // borrow of the VM state, and we can't pop things off while immutably borrowed.
            let foreign = state
                .foreign_functions
                .get(&ff_key(&self.lib, &self.name))
                .unwrap();
            let code_ptr = CodePtr::from_ptr(*foreign as *const _);
            match self.return_type {
                ItemType::Int(_) => Val::Int(
                    unsafe { call::<i32>(code_ptr, args.as_slice()) } as i64,
                ),
                ItemType::Char(_) => Val::Char(
                    unsafe { call::<u8>(code_ptr, args.as_slice()) } as char,
                ),
                ItemType::Bool(_) => Val::Bool(
                    unsafe { call::<i32>(code_ptr, args.as_slice()) } != 0,
                ),
                ItemType::Nil => {
                    unsafe { call::<()>(code_ptr, args.as_slice()) };
                    Val::Nil
                }
                _ => unreachable!(),
            }
        };

        // don't push nil values
        if !result.is_nil() {
            state.push(result);
        }
        Ok(())
    }

    pub(in vm) fn load(&self, state: &mut State) -> Result<()> {
        self.load_handle(state)?;
        self.load_function(state)?;
        Ok(())
    }

    /// Loads a function pointer into the current state.
    fn load_function(&self, state: &mut State) -> Result<()> {
        if !state.foreign_functions.contains_key(
            &ff_key(&self.lib, &self.name),
        )
        {
            let handle = state.dl_handles.get(&self.lib).unwrap();
            let fname = CString::new(self.name.clone()).unwrap();
            unsafe {
                let fun = libc::dlsym(*handle, fname.as_ptr());
                if fun.is_null() {
                    return Err(
                        format!(
                            "could not find symbol `{}` in dynamic library `{}`",
                            &self.name,
                            &self.lib
                        ).into(),
                    );
                }
                state.foreign_functions.insert(
                    ff_key(&self.lib, &self.name),
                    fun,
                );
            }
        }
        Ok(())
    }

    /// Loads a handle into the current state.
    fn load_handle(&self, state: &mut State) -> Result<()> {
        if !state.dl_handles.contains_key(&self.lib) {
            let lib_str = CString::new(self.lib.clone()).unwrap();
            unsafe {
                let handle = libc::dlopen(lib_str.as_ptr(), RTLD_NOW);
                if handle.is_null() {
                    return Err(
                        format!("could not open dynamic library `{}`", &self.lib).into(),
                    );
                }
                state.dl_handles.insert(self.lib.clone(), handle);
            }
        };
        Ok(())
    }
}
