use std::collections::HashMap;

use super::mmap::GuardedMmap;

/// The size of the evaluation stack in bytes.
const EVAL_STACK_SIZE: usize = 1024 * 1024;

/// An executable generated by the JIT compiler.
///
/// The executable contains the machine code generated by the JIT compiler and an evaluation stack.
pub struct Executable {
    /// The machine code to execute.
    pub code: Option<GuardedMmap>,
    /// The evaluation stack.
    pub eval_stack: Option<GuardedMmap>,
    // Mapping of variable names to their offsets in the variables area.
    pub variables_map: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum ExecutableError {
    CodeNotGenerated,
    CodeMemoryNotExecutable,
    EvalStackNotMmaped,
    UnknownVariable(String),
}

impl Executable {
    /// Create a new executable from the given machine code.
    ///
    /// # Arguments
    ///
    /// * `code_bytes` - The machine code to execute.
    ///
    /// # Returns
    ///
    /// A new executable.
    pub fn new(code_bytes: &[u8], variables_map: HashMap<String, usize>) -> Self {
        let mut code_page = GuardedMmap::new(code_bytes.len(), "code".to_string()).unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(
                code_bytes.as_ptr(),
                code_page.ptr() as *mut u8,
                code_bytes.len(),
            );
        }
        code_page.protect_rx().unwrap();

        Self {
            code: Some(code_page),
            eval_stack: Some(GuardedMmap::new(EVAL_STACK_SIZE, "eval_stack".to_string()).unwrap()),
            variables_map,
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

        // Prepare the variables area.
        for (name, value) in variables {
            match self.variables_map.get(name) {
                Some(offset) => {
                    variables_area[*offset] = *value;
                }
                None => {
                    println!("Skipping unknown variable: {}", name);
                }
            }
        }

        // unsafe { libc::getchar() };

        let func: extern "C" fn(*mut i64, *const i64) -> i64 =
            unsafe { std::mem::transmute(code_map.ptr()) };
        Ok(func(eval_stack_ptr as *mut i64, variables_area.as_ptr()))
    }
}