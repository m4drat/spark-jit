use std::collections::{HashMap, HashSet};

use crate::mmap::MmapBuf;

use super::mmap::GuardedMmap;

/// The size of the evaluation stack in bytes.
const EVAL_STACK_SIZE: usize = 16 * 1024;

/// An executable generated by the JIT compiler.
///
/// The executable contains the machine code generated by the JIT compiler and an evaluation stack.
pub struct Executable {
    /// The machine code to execute.
    pub code: Option<MmapBuf>,
    /// The evaluation stack.
    pub eval_stack: Option<MmapBuf>,
    // Mapping of variable names to their offsets in the variables area.
    pub variables_map: HashMap<String, usize>,
    // SHA256 hash of the code.
    pub integrity: Vec<u8>,
}

#[derive(Debug)]
pub enum ExecutableError {
    CodeNotGenerated,
    CodeMemoryNotExecutable,
    EvalStackNotMmaped,
    UnknownVariable(String),
    UnititializedVariable(String),
}

impl Executable {
    /// Create a new executable from the given machine code.
    ///
    /// # Arguments
    ///
    /// * `code_bytes` - The machine code to execute.
    /// * `integrity` - The SHA256 hash of the code (calculated by the compiler).
    /// * `variables_map` - Mapping of variable names to their offsets in the variables area.
    ///
    /// # Returns
    ///
    /// A new executable.
    pub fn new(code_bytes: &[u8], integrity: &[u8], variables_map: HashMap<String, usize>) -> Self {
        let mut stack_and_code = MmapBuf::new(EVAL_STACK_SIZE + code_bytes.len()).unwrap();
        stack_and_code.protect_rwx().unwrap();

        let code = stack_and_code.split_end(code_bytes.len()).unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(
                code_bytes.as_ptr(),
                code.ptr() as *mut u8,
                code_bytes.len(),
            );
        }

        Self {
            code: Some(code),
            eval_stack: Some(stack_and_code),
            variables_map,
            integrity: integrity.to_vec(),
        }
    }

    /// Run the executable.
    ///
    /// # Returns
    ///
    /// The result of the execution.
    pub fn run(&self, variables: &HashMap<String, i64>) -> Result<i64, ExecutableError> {
        // println!(
        //     "Eval stack ptr: {:p}",
        //     self.eval_stack.as_ref().unwrap().ptr()
        // );
        // println!("Code ptr: {:p}", self.code.as_ref().unwrap().ptr());

        let code_map = match &self.code {
            Some(code) => code,
            None => return Err(ExecutableError::CodeNotGenerated),
        };

        if !code_map.is_executable() {
            return Err(ExecutableError::CodeMemoryNotExecutable);
        }

        let eval_stack_ptr = match &self.eval_stack {
            Some(eval_stack) => eval_stack.ptr(),
            None => return Err(ExecutableError::EvalStackNotMmaped),
        };

        let mut variables_area = vec![0; self.variables_map.len()];
        let mut initialized_variables: HashSet<String> = HashSet::new();

        // Prepare the variables area.
        for (name, value) in variables {
            match self.variables_map.get(name) {
                Some(offset) => {
                    variables_area[*offset] = *value;
                    initialized_variables.insert(name.clone());
                }
                None => {
                    eprintln!(
                        "Variable with name {} was never used in the expression, skipping!",
                        name
                    );
                }
            }
        }

        // Check if all variables were initialized.
        for (name, _) in &self.variables_map {
            if !initialized_variables.contains(name) {
                return Err(ExecutableError::UnititializedVariable(name.clone()));
            }
        }

        let func: extern "C" fn(*mut i64, *const i64) -> i64 =
            unsafe { std::mem::transmute(code_map.ptr()) };
        Ok(func(eval_stack_ptr as *mut i64, variables_area.as_ptr()))
    }
}
