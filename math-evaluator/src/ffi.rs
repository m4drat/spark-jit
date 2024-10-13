use libc::{c_char, c_longlong, size_t};
use spark_jit::executable::Executable;

use crate::compiler;
use crate::rpn_converter;
use crate::tokenizer;
use std::collections::HashMap;
use std::ffi::c_void;

const KNOWN_VARIABLES: [&str; 4] = ["BALANCE", "STOCK_PRICE", "HOLDINGS", "HOLDINGS_VALUE"];

unsafe fn fill_error_buffer(output_error: *mut c_char, output_error_len: size_t, error: &str) {
    if output_error.is_null() {
        return;
    }

    let output_error = std::slice::from_raw_parts_mut(output_error as *mut u8, output_error_len);
    output_error[..output_error_len].copy_from_slice(error.as_bytes());
}

/// Compile the expression and return the executable that can be used to evaluate it with the given variables.
///
/// # Arguments
///
/// * `input` - The expression to compile.
/// * `output_error` - The buffer to write the error message to.
/// * * `output_error_len` - The length of the error buffer.
///
/// # Returns
///
/// The executable that can be used to evaluate the expression with the given variables. If the error occurs, the return value is null.
///
/// # Safety
///
/// The caller must ensure that the `output_error` buffer is valid and has the length of at least `output_error_len`.
pub unsafe extern "C" fn compile_expression(
    input: *const c_char,
    output_error: *mut c_char,
    output_error_len: size_t,
) -> *mut c_void {
    if input.is_null() {
        unsafe {
            fill_error_buffer(
                output_error,
                output_error_len,
                "Invalid input string pointer!",
            );
        }
        return std::ptr::null_mut();
    }

    let input = match unsafe { std::ffi::CStr::from_ptr(input).to_str() } {
        Ok(input) => input,
        Err(_) => {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    "Failed to convert the input string to a Rust string!",
                )
            }
            return std::ptr::null_mut();
        }
    };

    let mut tokenizer = tokenizer::Tokenizer::new();
    let tokens = match tokenizer.tokenize(input) {
        Ok(tokens) => tokens,
        Err(e) => {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    &format!("Failed to tokenize the input: {}", e),
                );
            }
            return std::ptr::null_mut();
        }
    };

    for variable in tokenizer.get_variables() {
        if !KNOWN_VARIABLES.contains(&variable.as_str()) {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    &format!(
                        "The input expression contains unknown variable: {}",
                        variable
                    ),
                );
            }
            return std::ptr::null_mut();
        }
    }

    let rpn = match rpn_converter::RpnConverter::convert(&tokens) {
        Ok(tokens) => tokens,
        Err(e) => {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    &format!("Failed to convert the input to RPN: {}", e),
                );
            }
            return std::ptr::null_mut();
        }
    };

    let mut compiler = compiler::Compiler::new();
    let exe = match compiler.compile(&rpn) {
        Ok(exe) => exe,
        Err(e) => {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    &format!("Failed to compile the RPN expression: {}", e),
                );
            }
            return std::ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(exe)) as *mut c_void
}

/// Evaluate the expression with the given variables.
///
/// # Arguments
///
/// * `exe` - The executable to use as the evaluator.
/// * `keys_ptr` - The pointer to the array of variable names.
/// * `values_ptr` - The pointer to the array of variable values.
/// * `variables_len` - The number of variables.
/// * `output_error` - The buffer to write the error message to.
/// * `output_error_len` - The length of the error buffer.
///
/// # Returns
///
/// The result of the evaluation. If the error-buffer is non-empty, the return value is set to 0.
///
/// # Safety
///
/// The caller must ensure that the `exe` pointer is valid and was not freed before. The `keys_ptr` and `values_ptr` must be valid pointers to the arrays of the same length. The `output_error` buffer must be valid and have the length of at least `output_error_len`.
pub unsafe extern "C" fn evaluate_expression(
    exe: *mut c_void,
    keys_ptr: *const c_char,
    values_ptr: *const c_longlong,
    variables_len: size_t,
    output_error: *mut c_char,
    output_error_len: size_t,
) -> c_longlong {
    if exe.is_null() {
        unsafe {
            fill_error_buffer(
                output_error,
                output_error_len,
                "Invalid executable pointer!",
            );
        }
        return 0;
    }

    if keys_ptr.is_null() {
        unsafe {
            fill_error_buffer(output_error, output_error_len, "Invalid variables pointer!");
        }
        return 0;
    }

    let keys = unsafe {
        std::slice::from_raw_parts(keys_ptr, variables_len)
            .iter()
            .map(|key| std::ffi::CStr::from_ptr(key).to_str().unwrap())
            .collect::<Vec<&str>>()
    };

    let values = unsafe { std::slice::from_raw_parts(values_ptr, variables_len) };

    let mut variables = HashMap::new();
    for (key, value) in keys.iter().zip(values.iter()) {
        variables.insert(key.to_string(), *value);
    }

    let exe = unsafe { &*(exe as *const Executable) };
    match exe.run(&variables) {
        Ok(result) => result,
        Err(e) => {
            unsafe {
                fill_error_buffer(
                    output_error,
                    output_error_len,
                    &format!("Failed to evaluate the expression: {:?}", e),
                );
            }
            0
        }
    }
}

/// Free the executable.
///
/// # Arguments
///
/// * `exe` - The executable to free.
///
/// # Safety
///
/// The caller must ensure that the `exe` pointer is valid and was not freed before.
pub unsafe extern "C" fn free_executable(exe: *mut c_void) {
    if exe.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(exe as *mut Executable));
    }
}
