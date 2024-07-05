use crate::tokenizer::{Token, TokenizedInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RPNConverterError {
    MismatchedClosingParen,
    MismatchedOpeningParen,
    NotEnoughOperands,
    TooManyOperands,
}

impl std::fmt::Display for RPNConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RPNConverterError::MismatchedClosingParen => {
                write!(f, "Mismatched closing parenthesis")
            }
            RPNConverterError::MismatchedOpeningParen => {
                write!(f, "Mismatched opening parenthesis")
            }
            RPNConverterError::NotEnoughOperands => write!(f, "Not enough operands"),
            RPNConverterError::TooManyOperands => write!(f, "Too many operands"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RPNExpr(pub Vec<Token>);

impl std::ops::Deref for RPNExpr {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RPNExpr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct RpnConverter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
    NA,
}

struct PrecAssoc {
    pub prec: i32,
    pub assoc: Associativity,
}

impl RpnConverter {
    /// Returns the precedence and associativity of the operator.
    fn get_prec_assoc(op: &Token) -> PrecAssoc {
        use crate::tokenizer::Op::*;
        use crate::tokenizer::Token::*;
        use Associativity::*;

        match *op {
            BinaryOp(Minus) | BinaryOp(Plus) => PrecAssoc {
                prec: 1,
                assoc: Left,
            },
            BinaryOp(Mult) | BinaryOp(Div) => PrecAssoc {
                prec: 2,
                assoc: Left,
            },
            BinaryOp(Pow) => PrecAssoc {
                prec: 4,
                assoc: Right,
            },
            UnaryOp(Minus) | UnaryOp(Plus) => PrecAssoc { prec: 3, assoc: NA },
            UnaryOp(Fact) => PrecAssoc { prec: 5, assoc: NA },
            _ => PrecAssoc { prec: 0, assoc: NA },
        }
    }

    /// Verifies that the RPN expression is valid.
    fn verify_rpn(tokens: &[Token]) -> Result<(), RPNConverterError> {
        use crate::tokenizer::Token::*;
        let mut n_operands = 0isize;
        for token in tokens {
            match token {
                Number(_) | Variable(_) => n_operands += 1,
                BinaryOp(_) => {
                    n_operands -= 1;
                }
                UnaryOp(_) => (),
                _ => panic!("Unexpected token in RPN"),
            }

            if n_operands < 1 {
                return Err(RPNConverterError::NotEnoughOperands);
            }
        }

        if n_operands > 1 {
            return Err(RPNConverterError::TooManyOperands);
        }

        Ok(())
    }

    /// Converts infix notation to Reverse Polish Notation
    /// using the Shunting Yard algorithm.
    pub fn convert(tokens: &TokenizedInput) -> Result<RPNExpr, RPNConverterError> {
        use crate::tokenizer::Token::*;
        let mut output = Vec::new();
        let mut stack = Vec::new();

        for token in tokens.iter() {
            let token = token.clone();
            match token {
                Number(_) | Variable(_) => output.push(token),
                UnaryOp(_) => stack.push(token),
                BinaryOp(_) => {
                    let pa1 = RpnConverter::get_prec_assoc(&token);
                    while !stack.is_empty() {
                        let pa2 = RpnConverter::get_prec_assoc(stack.last().unwrap());
                        if (pa1.assoc == Associativity::Left && pa1.prec <= pa2.prec)
                            || (pa1.assoc == Associativity::Right && pa1.prec < pa2.prec)
                        {
                            output.push(stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }

                    stack.push(token);
                }
                LParen => stack.push(token),
                RParen => {
                    let mut found = false;
                    while let Some(tok) = stack.pop() {
                        if tok == LParen {
                            found = true;
                            break;
                        }
                        output.push(tok);
                    }

                    if !found {
                        return Err(RPNConverterError::MismatchedClosingParen);
                    }
                }
            }
        }

        while let Some(tok) = stack.pop() {
            match tok {
                BinaryOp(_) | UnaryOp(_) => output.push(tok),
                LParen => return Err(RPNConverterError::MismatchedOpeningParen),
                _ => panic!("Unexpected token on the stack"),
            }
        }

        match RpnConverter::verify_rpn(&output) {
            Ok(_) => Ok(RPNExpr(output)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Op::*;

    #[test]
    fn test_rpn_converter() {
        let tokens = TokenizedInput(vec![
            Token::Number(1),
            Token::BinaryOp(Plus),
            Token::Number(2),
            Token::BinaryOp(Mult),
            Token::Number(3),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(1),
                Token::Number(2),
                Token::Number(3),
                Token::BinaryOp(Mult),
                Token::BinaryOp(Plus),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_parentheses_1() {
        let tokens = TokenizedInput(vec![
            Token::LParen,
            Token::Number(1),
            Token::BinaryOp(Plus),
            Token::Number(2),
            Token::RParen,
            Token::BinaryOp(Mult),
            Token::Number(3),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(1),
                Token::Number(2),
                Token::BinaryOp(Plus),
                Token::Number(3),
                Token::BinaryOp(Mult),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_parentheses_2() {
        let tokens = TokenizedInput(vec![
            Token::LParen,
            Token::Number(3),
            Token::BinaryOp(Minus),
            Token::Number(1),
            Token::RParen,
            Token::BinaryOp(Mult),
            Token::Number(2),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(3),
                Token::Number(1),
                Token::BinaryOp(Minus),
                Token::Number(2),
                Token::BinaryOp(Mult),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_unary() {
        let tokens = TokenizedInput(vec![
            Token::UnaryOp(Minus),
            Token::Number(1),
            Token::BinaryOp(Plus),
            Token::Number(2),
            Token::BinaryOp(Mult),
            Token::Number(3),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(1),
                Token::UnaryOp(Minus),
                Token::Number(2),
                Token::Number(3),
                Token::BinaryOp(Mult),
                Token::BinaryOp(Plus),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_associativity() {
        let tokens = TokenizedInput(vec![
            Token::Number(1),
            Token::BinaryOp(Plus),
            Token::Number(2),
            Token::BinaryOp(Mult),
            Token::Number(3),
            Token::BinaryOp(Mult),
            Token::Number(4),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(1),
                Token::Number(2),
                Token::Number(3),
                Token::BinaryOp(Mult),
                Token::Number(4),
                Token::BinaryOp(Mult),
                Token::BinaryOp(Plus),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_associativity_2() {
        let tokens = TokenizedInput(vec![
            Token::Number(3),
            Token::UnaryOp(Fact),
            Token::BinaryOp(Plus),
            Token::Number(5),
            Token::UnaryOp(Fact),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(3),
                Token::UnaryOp(Fact),
                Token::Number(5),
                Token::UnaryOp(Fact),
                Token::BinaryOp(Plus),
            ])
        );
    }

    #[test]
    fn test_rpn_converter_associativity_3_exp() {
        let tokens = TokenizedInput(vec![
            Token::Number(2),
            Token::BinaryOp(Pow),
            Token::Number(3),
            Token::BinaryOp(Pow),
            Token::Number(4),
        ]);

        assert_eq!(
            RpnConverter::convert(&tokens).unwrap(),
            RPNExpr(vec![
                Token::Number(2),
                Token::Number(3),
                Token::Number(4),
                Token::BinaryOp(Pow),
                Token::BinaryOp(Pow),
            ])
        );
    }
}
