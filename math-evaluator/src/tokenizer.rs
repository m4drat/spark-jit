use std::{collections::HashSet, iter};

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Plus,
    Minus,
    Mult,
    Div,
    Fact,
    Pow,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Variable(String),
    Number(i64),
    UnaryOp(Op),
    BinaryOp(Op),
    LParen,
    RParen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerError {
    IntegerParseError,
    UnexpectedCharacter(char),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenizerError::IntegerParseError => write!(f, "Failed to parse integer"),
            TokenizerError::UnexpectedCharacter(c) => write!(f, "Unexpected character: '{}'", c),
        }
    }
}

#[derive(Default)]
pub struct Tokenizer {
    prev: Option<Token>,
    variables: HashSet<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenizedInput(pub Vec<Token>);

impl std::ops::Deref for TokenizedInput {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TokenizedInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            prev: None,
            variables: HashSet::new(),
        }
    }

    fn makes_unary(&self) -> bool {
        !matches!(
            self.prev,
            Some(Token::Number(_)) | Some(Token::Variable(_)) | Some(Token::RParen)
        )
    }

    pub fn get_variables(&self) -> &HashSet<String> {
        &self.variables
    }

    pub fn tokenize(&mut self, input: &str) -> Result<TokenizedInput, TokenizerError> {
        use Token::*;

        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let var: String = iter::once(c)
                        .chain(iter::from_fn(|| {
                            chars
                                .by_ref()
                                .next_if(|c| c.is_ascii_alphanumeric() || *c == '_')
                        }))
                        .collect();
                    self.variables.insert(var.clone());
                    tokens.push(Variable(var));
                }
                '0'..='9' => {
                    let pred = if let Some('x') = chars.peek() {
                        chars.next();
                        char::is_ascii_hexdigit
                    } else {
                        char::is_ascii_digit
                    };

                    let num: i64 = i64::from_str_radix(
                        iter::once(c)
                            .chain(iter::from_fn(|| chars.by_ref().next_if(pred)))
                            .collect::<String>()
                            .as_str(),
                        if pred == char::is_ascii_digit { 10 } else { 16 },
                    )
                    .map_err(|_| TokenizerError::IntegerParseError)?;
                    tokens.push(Number(num));
                }
                '+' => tokens.push(BinaryOp(Op::Plus)),
                '-' => {
                    if self.makes_unary() {
                        tokens.push(UnaryOp(Op::Minus));
                    } else {
                        tokens.push(BinaryOp(Op::Minus));
                    }
                }
                '!' => tokens.push(UnaryOp(Op::Fact)),
                '^' => tokens.push(BinaryOp(Op::Pow)),
                '*' => tokens.push(BinaryOp(Op::Mult)),
                '/' => tokens.push(BinaryOp(Op::Div)),
                '(' => tokens.push(LParen),
                ')' => tokens.push(RParen),
                c if c.is_whitespace() => {}
                _ => {
                    return Err(TokenizerError::UnexpectedCharacter(c));
                }
            }

            self.prev = tokens.last().cloned();
        }

        Ok(TokenizedInput(tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let mut tokenizer = Tokenizer::new();
        assert_eq!(
            tokenizer
                .tokenize("((17132123123 + 123123) * ( -1337 ^  - 4  )) / 5!")
                .unwrap(),
            TokenizedInput(vec![
                Token::LParen,
                Token::LParen,
                Token::Number(17132123123),
                Token::BinaryOp(Op::Plus),
                Token::Number(123123),
                Token::RParen,
                Token::BinaryOp(Op::Mult),
                Token::LParen,
                Token::UnaryOp(Op::Minus),
                Token::Number(1337),
                Token::BinaryOp(Op::Pow),
                Token::UnaryOp(Op::Minus),
                Token::Number(4),
                Token::RParen,
                Token::RParen,
                Token::BinaryOp(Op::Div),
                Token::Number(5),
                Token::UnaryOp(Op::Fact),
            ])
        );
    }

    #[test]
    fn test_tokenizer_unexpected_char() {
        let mut tokenizer = Tokenizer::new();
        assert_eq!(
            tokenizer.tokenize("1 + 2 * 3 - 4 / 5 b").unwrap_err(),
            TokenizerError::UnexpectedCharacter('b')
        );
    }

    #[test]
    fn test_tokenizer_integer_parse_error() {
        let mut tokenizer = Tokenizer::new();
        assert_eq!(
            tokenizer
                .tokenize("1 + 2 * 3123123123123123123123 - 4 / 5 6")
                .unwrap_err(),
            TokenizerError::IntegerParseError
        );
    }
}
