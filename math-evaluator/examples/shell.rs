#![feature(extract_if)]

use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use math_evaluator::compiler::Compiler;
use math_evaluator::rpn_converter::RpnConverter;
use math_evaluator::tokenizer::{self, Tokenizer};

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
    println!("Welcome to the Calculator as a Service (CaaS)!");
    println!("Please enter an expression to evaluate:");

    // Read file input.txt
    let mut input = String::new();
    let mut file = std::fs::File::open("input.txt").unwrap();
    file.read_to_string(&mut input).unwrap();

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

    println!("RPN: {:?}", rpn);

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

        if tokenizer.get_variables().is_empty() {
            unsafe { libc::getchar() };
        }
    }

    // if result_interpreter != result_compiled {
    //     eprintln!("The interpreter and JIT results do not match!");
    //     eprintln!("Interpreter: {}", result_interpreter);
    //     eprintln!("JIT        : {}", result_compiled);
    //     panic!("Results do not match");
    // }

    // println!("Thank you for using the Calculator as a Service (CaaS)!");
}
