#![feature(extract_if)]

use std::collections::HashMap;
use std::env;

use math_evaluator::compiler::Compiler;
use math_evaluator::rpn_converter::RpnConverter;
use math_evaluator::tokenizer::Tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <expression>", args[0]);
        return;
    }

    let input = args[1].clone();

    let mut tokenizer = Tokenizer::new();
    let tokens = match tokenizer.tokenize(&input) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to tokenize the input: {}", e);
            return;
        }
    };

    let rpn = match RpnConverter::convert(&tokens) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to convert the input to RPN: {}", e);
            return;
        }
    };

    let mut compiler = Compiler::new();
    let exe = match compiler.compile(&rpn) {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("Failed to compile the RPN expression: {}", e);
            return;
        }
    };

    println!("Code integrity: {}", hex::encode(&exe.integrity));

    let mut vars: HashMap<String, i64> = HashMap::new();
    vars.insert("HOLDINGS_VALUE".to_string(), 1);
    vars.insert("STOCK_PRICE".to_string(), 1);
    vars.insert("BALANCE".to_string(), 1);
    vars.insert("HOLDINGS".to_string(), 1);
    let result = match exe.run(&vars) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to run the compiled code: {:?}", e);
            return;
        }
    };

    println!("Result: {}", result);
}
