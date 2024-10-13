use std::collections::HashMap;

use crate::rpn_converter::RPNExpr;
pub struct RpnEvaluator {}

#[derive(Debug)]
pub enum RpnEvaluatorError {
    UnknownVariable(String),
}

impl std::fmt::Display for RpnEvaluatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RpnEvaluatorError::UnknownVariable(name) => {
                write!(f, "Unknown variable: {}", name)
            }
        }
    }
}

impl RpnEvaluator {
    pub fn evaluate(
        tokens: &RPNExpr,
        variables: &HashMap<String, i64>,
    ) -> Result<i64, RpnEvaluatorError> {
        use crate::tokenizer::Op::*;
        use crate::tokenizer::Token::*;

        let mut eval_stack: Vec<i64> = vec![];

        for token in tokens.iter() {
            match token {
                Variable(name) => {
                    if let Some(value) = variables.get(name) {
                        eval_stack.push(*value);
                    } else {
                        return Err(RpnEvaluatorError::UnknownVariable(name.clone()));
                    }
                }
                Number(num) => {
                    eval_stack.push(*num);
                }
                BinaryOp(op) => {
                    let b = eval_stack.pop().unwrap();
                    let a = eval_stack.pop().unwrap();
                    let result = match op {
                        Plus => a + b,
                        Minus => a - b,
                        Mult => a * b,
                        Div => a / b,
                        Pow => a.pow(b as u32),
                        _ => panic!("Unexpected binary operator"),
                    };
                    eval_stack.push(result);
                }
                UnaryOp(op) => {
                    let a = eval_stack.pop().unwrap();
                    let result = match op {
                        Minus => -a,
                        Plus => a,
                        Fact => {
                            let mut result = 1;
                            for i in 1..=a {
                                result *= i;
                            }
                            result
                        }
                        _ => panic!("Unexpected unary operator"),
                    };
                    eval_stack.push(result);
                }
                _ => {}
            }
        }

        if eval_stack.len() != 1 {
            panic!("Invalid RPN expression");
        }

        Ok(eval_stack[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Op::*;
    use crate::tokenizer::Token::*;

    #[test]
    fn test_rpn_evaluator() {
        let tokens = RPNExpr(vec![Number(1), Number(2), BinaryOp(Plus)]);
        assert_eq!(RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(), 3);

        let tokens = RPNExpr(vec![Number(1), Number(2), BinaryOp(Minus)]);
        assert_eq!(
            RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(),
            -1
        );

        let tokens = RPNExpr(vec![Number(2), Number(3), BinaryOp(Mult)]);
        assert_eq!(RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(), 6);

        let tokens = RPNExpr(vec![Number(6), Number(3), BinaryOp(Div)]);
        assert_eq!(RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(), 2);

        let tokens = RPNExpr(vec![Number(1), Number(2), UnaryOp(Minus), BinaryOp(Plus)]);
        assert_eq!(
            RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(),
            -1
        );

        let tokens = RPNExpr(vec![Number(1), Number(2), UnaryOp(Minus), BinaryOp(Minus)]);
        assert_eq!(RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(), 3);

        let tokens = RPNExpr(vec![Number(1), Number(2), UnaryOp(Minus), BinaryOp(Mult)]);
        assert_eq!(
            RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(),
            -2
        );

        let tokens = RPNExpr(vec![Number(1), Number(2), UnaryOp(Minus), BinaryOp(Div)]);
        assert_eq!(RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(), 0);

        let tokens = RPNExpr(vec![Number(5), UnaryOp(Fact)]);
        assert_eq!(
            RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(),
            120
        );

        let tokens = RPNExpr(vec![Number(5), Number(2), BinaryOp(Pow)]);
        assert_eq!(
            RpnEvaluator::evaluate(&tokens, &HashMap::new()).unwrap(),
            25
        );
    }

    #[test]
    fn test_convert_to_rpn_and_eval() {
        let input = "((123 * 6 + 123123) * ( -1337 --- -4 )) * 5 / 120";
        let mut tokenizer = crate::tokenizer::Tokenizer::new();
        let tokens = tokenizer.tokenize(input).unwrap();
        let rpn = crate::rpn_converter::RpnConverter::convert(&tokens).unwrap();
        assert_eq!(
            RpnEvaluator::evaluate(&rpn, &HashMap::new()).unwrap(),
            -6879446
        );

        let input = "1 + 1 + 15 * 3 - 1 - -2";
        let mut tokenizer = crate::tokenizer::Tokenizer::new();
        let tokens = tokenizer.tokenize(input).unwrap();
        let rpn = crate::rpn_converter::RpnConverter::convert(&tokens).unwrap();
        assert_eq!(RpnEvaluator::evaluate(&rpn, &HashMap::new()).unwrap(), 48);

        let input = "(-19 + (7! + -1 * (9724 + 82402)) * (3 - 812 - (13 - 7)!)) / 4";
        let mut tokenizer = crate::tokenizer::Tokenizer::new();
        let tokens = tokenizer.tokenize(input).unwrap();
        let rpn = crate::rpn_converter::RpnConverter::convert(&tokens).unwrap();
        assert_eq!(
            RpnEvaluator::evaluate(&rpn, &HashMap::new()).unwrap(),
            33288618
        );

        let input = "((-2 ^ 3) ^ 4) * (3 ^ 2) - 1";
        let mut tokenizer = crate::tokenizer::Tokenizer::new();
        let tokens = tokenizer.tokenize(input).unwrap();
        let rpn = crate::rpn_converter::RpnConverter::convert(&tokens).unwrap();
        assert_eq!(
            RpnEvaluator::evaluate(&rpn, &HashMap::new()).unwrap(),
            36863
        );
    }
}
