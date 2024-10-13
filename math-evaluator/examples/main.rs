#![feature(extract_if)]

use std::{
    collections::HashMap,
    io::{self, Write},
};

use math_evaluator::compiler::Compiler;
use math_evaluator::rpn_converter::RpnConverter;
use math_evaluator::rpn_evaluator::RpnEvaluator;
use math_evaluator::tokenizer::{self, Tokenizer};

// fn sigfpe_handler(_sig: i32, info: *mut libc::siginfo_t, _ucontext: *mut libc::c_void) {
//     let info = unsafe { *info };
//     let code = info.si_code;
//     let addr = unsafe { info.si_addr() };
//     println!(
//         "Caught Zero-Division error at address {:p}, code {}",
//         addr, code
//     );

//     // Update the context to "skip" the faulting instruction
//     // This is actually, where the bug is introduced. Usually, the `div` instruction
//     // is 2 bytes long, but if its operand is the register of 64-bit size, the instruction is 3 bytes long (+REX prefix).
//     let ucontext = unsafe { &mut *(_ucontext as *mut libc::ucontext_t) };
//     ucontext.uc_mcontext.gregs[libc::REG_RIP as usize] += 2;

//     // Set the result of the division to 0
//     // ucontext.uc_mcontext.gregs[libc::REG_RAX as usize] = 0;
//     // ucontext.uc_mcontext.gregs[libc::REG_RDX as usize] = 0;
// }

// fn setup_sigfpe_handler() {
//     unsafe {
//         let mut sa: libc::sigaction = std::mem::zeroed();
//         sa.sa_flags = libc::SA_SIGINFO | libc::SA_NODEFER;
//         sa.sa_sigaction = sigfpe_handler as usize;
//         libc::sigaction(libc::SIGFPE, &sa, std::ptr::null_mut());
//     }
// }

fn pretty_print_expr(
    tokens: &Vec<tokenizer::Token>,
    variables: &HashMap<String, i64>,
    expr_result: i64,
) -> String {
    use tokenizer::Op;
    use tokenizer::Token::*;
    let mut result = String::new();

    // Dump variables
    for (name, value) in variables {
        result.push_str(&format!("{} = {}\n", name, value).to_string());
    }

    for token in tokens {
        match token {
            Number(n) => result.push_str(&n.to_string()),
            BinaryOp(op) => match op {
                Op::Plus => result.push_str("+"),
                Op::Minus => result.push_str("-"),
                Op::Mult => result.push_str("*"),
                Op::Div => result.push_str("/"),
                Op::Pow => result.push_str("^"),
                Op::Fact => result.push_str("!"),
            },
            UnaryOp(op) => match op {
                Op::Plus => result.push_str("+"),
                Op::Minus => result.push_str("-"),
                Op::Fact => result.push_str("!"),
                _ => panic!("Unexpected unary operator"),
            },
            LParen => result.push_str("("),
            RParen => result.push_str(")"),
            Variable(name) => result.push_str(name), // result.push_str(&format!("{}${}", name, variables[name]).to_string())
        }
        result.push(' ');
    }

    result.push_str(&format!("= {}", expr_result).to_string());

    result
}

fn main() {
    // setup_sigfpe_handler();

    println!("Welcome to the Calculator as a Service (CaaS)!");
    println!("Please enter an expression to evaluate:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mut tokenizer = Tokenizer::new();
    let tokens = match tokenizer.tokenize(&input) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to tokenize the input: {}", e);
            return;
        }
    };

    println!("Tokens: {:?}", tokens);

    let rpn = match RpnConverter::convert(&tokens) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to convert the input to RPN: {}", e);
            return;
        }
    };

    // println!("RPN: {:?}", rpn);

    // let result_interpreter = match RpnEvaluator::evaluate(&rpn, &HashMap::new()) {
    //     Ok(result) => result,
    //     Err(e) => {
    //         eprintln!("Failed to evaluate the RPN expression: {}", e);
    //         return;
    //     }
    // };
    // println!("Result (interpreter) : {}", result_interpreter);

    let mut compiler = Compiler::new();
    let exe = match compiler.compile(&rpn) {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("Failed to compile the RPN expression: {}", e);
            return;
        }
    };

    loop {
        let mut variables = HashMap::new();

        for name in tokenizer.get_variables() {
            print!("Please enter the value for variable '{}': ", name);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value: i64 = match input.trim().parse() {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Failed to parse the input: {}", e);
                    return;
                }
            };
            variables.insert(name.clone(), value);
        }

        let result_compiled = match exe.run(&variables) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to run the compiled code: {:?}", e);
                return;
            }
        };
        println!(
            "{}",
            pretty_print_expr(&tokens, &variables, result_compiled)
        );
    }

    // if result_interpreter != result_compiled {
    //     eprintln!("The interpreter and JIT results do not match!");
    //     eprintln!("Interpreter: {}", result_interpreter);
    //     eprintln!("JIT        : {}", result_compiled);
    //     panic!("Results do not match");
    // }

    // println!("Thank you for using the Calculator as a Service (CaaS)!");
}
