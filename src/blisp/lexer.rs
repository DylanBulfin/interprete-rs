use crate::error::InterpreteResult;

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralSuffix {
    None,
    Unsigned,
    Float,
    Char,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NumLiteral {
    negative: bool,
    int_part: u64,
    float: bool,
    dec_part: u64,
    suffix: LiteralSuffix,
}

pub enum ReservedIdent {
    // Math
    Add,
    Sub,
    Div,
    Mul,

    // I/O
    Write,
    Read,

    // Control flow
    If,
    While,

    // Boolean ops
    Eq,
    Neq,
    Leq,
    Geq,
    Lt,
    Gt,
    And,
    Or,

    // Vars
    Set,
    Init,
    Def,

    // Collections
    Concat,
    Prepend,
    Take,
    Split,

    // Convenience
    Eval,
    ToString,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    UInt,
    Float,
    List(Box<Type>),
    Tuple(Box<Type>, Box<Type>),
    Unit,
    Char,
    Bool,
    // String, // Probably want to leave out until a need arises, not sure if useful
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    NumLiteral(NumLiteral),
    CharLiteral(u8),
    UnitLiteral,
    String(String),
    Ident(String),
    Type(Type),
    Plus,
    Minus,
    Div,
    Mult,
    Comma,
    LParen,
    RParen,
    LBrack,
    RBrack,
    SingleQuote,
    DoubleQuote,
    EOF,
}

fn handle_num_literal(input: &[char]) -> InterpreteResult<(NumLiteral, usize)> {
    macro_rules! error_template {
        () => {
            "Unexpected char while parsing number: {}"
        };
    }

    let mut curr_index = 0;

    let negative = input[0] == '-';
    if negative {
        curr_index += 1;
    }

    // This check explicitly ensures we have a digit at the start of the number before the real
    // parsing
    let mut int_part = input[curr_index]
        .to_digit(10)
        .ok_or(format!(error_template!(), input[curr_index]))? as u64;
    curr_index += 1;

    let mut float = false;
    let mut dec_part = 0;
    let mut suffix = LiteralSuffix::None;

    loop {
        if curr_index >= input.len() {
            break;
        }
        match input[curr_index] {
            '0'..='9' => {
                if float {
                    dec_part *= 10;
                    dec_part += input[curr_index].to_digit(10).unwrap() as u64;
                } else {
                    int_part *= 10;
                    int_part += input[curr_index].to_digit(10).unwrap() as u64;
                }

                curr_index += 1;
            }
            '.' => {
                float = true;
                curr_index += 1;
            }
            'u' => {
                suffix = LiteralSuffix::Unsigned;
                curr_index += 1;
                break;
            }
            'f' => {
                suffix = LiteralSuffix::Float;
                curr_index += 1;
                break;
            }
            'c' => {
                suffix = LiteralSuffix::Char;
                curr_index += 1;
                break;
            }
            _ => break,
        }
    }

    Ok((
        NumLiteral {
            int_part,
            dec_part,
            float,
            suffix,
            negative,
        },
        curr_index,
    ))
}

pub fn tokenize(input: Vec<char>) -> InterpreteResult<Vec<Token>> {
    let mut curr_index = 0;
    let mut res = Vec::new();

    loop {
        if curr_index >= input.len() {
            break;
        }
        res.push(match input[curr_index] {
            '+' => Token::Plus,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '/' => Token::Div,
            '*' => Token::Mult,
            '\'' => Token::SingleQuote,
            '\"' => Token::DoubleQuote,
            '0'..='9' => {
                let (lit, count) = handle_num_literal(&input[curr_index..])?;
                curr_index += count - 1;
                Token::NumLiteral(lit)
            }
            '-' => {
                if input
                    .get(curr_index + 1)
                    .ok_or("Unexpectedly reached end of input")?
                    == &' '
                {
                    Token::Minus
                } else {
                    let (lit, count) = handle_num_literal(&input[curr_index..])?;
                    curr_index += count - 1;
                    Token::NumLiteral(lit)
                }
            }
            _ => unimplemented!(),
        });

        curr_index += 1;
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::error::InterpreTestResult;

    use super::*;

    #[test]
    fn parentheses() -> InterpreTestResult {
        let (input1, output1) = (
            "(())".chars().collect(),
            vec![Token::LParen, Token::LParen, Token::RParen, Token::RParen],
        );
        let (input2, output2) = (
            "((())".chars().collect(),
            vec![
                Token::LParen,
                Token::LParen,
                Token::LParen,
                Token::RParen,
                Token::RParen,
            ],
        );

        assert_eq!(tokenize(input1)?, output1);
        assert_eq!(tokenize(input2)?, output2);

        Ok(())
    }

    #[test]
    fn num_literals() -> InterpreTestResult {
        let (input1, output1) = (
            "(1243)".chars().collect(),
            vec![
                Token::LParen,
                Token::NumLiteral(NumLiteral {
                    int_part: 1243,
                    float: false,
                    negative: false,
                    dec_part: 0,
                    suffix: LiteralSuffix::None,
                }),
                Token::RParen,
            ],
        );
        let (input2, output2) = (
            "(-124.3)".chars().collect(),
            vec![
                Token::LParen,
                Token::NumLiteral(NumLiteral {
                    int_part: 124,
                    float: true,
                    negative: true,
                    dec_part: 3,
                    suffix: LiteralSuffix::None,
                }),
                Token::RParen,
            ],
        );
        let (input3, output3) = (
            "(-124f)".chars().collect(),
            vec![
                Token::LParen,
                Token::NumLiteral(NumLiteral {
                    int_part: 124,
                    float: false,
                    negative: true,
                    dec_part: 0,
                    suffix: LiteralSuffix::Float,
                }),
                Token::RParen,
            ],
        );

        assert_eq!(tokenize(input1)?, output1);
        assert_eq!(tokenize(input2)?, output2);
        assert_eq!(tokenize(input3)?, output3);

        Ok(())
    }
}
