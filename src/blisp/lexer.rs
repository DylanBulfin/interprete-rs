use crate::error::InterpreteResult;

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralSuffix {
    None,
    Unsigned,
    Float,
    Char,
}

impl From<char> for LiteralSuffix {
    fn from(value: char) -> Self {
        match value {
            'c' => LiteralSuffix::Char,
            'u' => LiteralSuffix::Unsigned,
            'f' => LiteralSuffix::Float,
            _ => panic!(
                "Attempted to convert invalid char {} to LiteralSuffix",
                value
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NumLiteral {
    negative: bool,
    int_part: u64,
    float: bool,
    dec_part: u64,
    suffix: LiteralSuffix,
}

impl NumLiteral {
    pub fn new_int(int_part: u64, negative: bool) -> Self {
        Self {
            int_part,
            negative,
            suffix: LiteralSuffix::None,
            dec_part: 0,
            float: false,
        }
    }

    pub fn new_int_with_suffix(int_part: u64, negative: bool, suffix: char) -> Self {
        Self {
            int_part,
            negative,
            suffix: suffix.into(),
            dec_part: 0,
            float: false,
        }
    }

    pub fn new_float(int_part: u64, dec_part: u64, negative: bool) -> Self {
        Self {
            int_part,
            dec_part,
            negative,
            float: true,
            suffix: LiteralSuffix::None,
        }
    }

    pub fn new_float_with_suffix(
        int_part: u64,
        dec_part: u64,
        negative: bool,
        suffix: char,
    ) -> Self {
        Self {
            int_part,
            dec_part,
            negative,
            float: true,
            suffix: suffix.into(),
        }
    }
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

impl From<NumLiteral> for Token {
    fn from(value: NumLiteral) -> Self {
        Self::NumLiteral(value)
    }
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

        match input[curr_index] {
            '+' => res.push(Token::Plus),
            '(' => res.push(Token::LParen),
            ')' => res.push(Token::RParen),
            '/' => res.push(Token::Div),
            '*' => res.push(Token::Mult),
            '\'' => res.push(Token::SingleQuote),
            '\"' => res.push(Token::DoubleQuote),
            '0'..='9' => {
                let (lit, count) = handle_num_literal(&input[curr_index..])?;
                curr_index += count - 1;
                res.push(Token::NumLiteral(lit))
            }
            '-' => {
                if input
                    .get(curr_index + 1)
                    .ok_or("Unexpectedly reached end of input")?
                    == &' '
                {
                    res.push(Token::Minus);
                } else {
                    let (lit, count) = handle_num_literal(&input[curr_index..])?;
                    curr_index += count - 1;
                    res.push(Token::NumLiteral(lit));
                }
            }
            ' ' => (),
            _ => unimplemented!(),
        };

        curr_index += 1;
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::error::InterpreTestResult;

    use super::*;

    // This is an attempt at a macro to make forming numeric literals easier but it's a complete
    // nightmare
    macro_rules! num_lit {
        ($(-1$neg:vis,;)? $int:literal $(,$float:literal)? $(;$suffix:ident)?) => {
            #[allow(unused_mut)]
            {
                let mut negative = false;
                let mut int_part = $int;
                let mut float = false;
                let mut dec_part = 0;
                let mut suffix = LiteralSuffix::None;

                $(
                    $ignore($neg);
                    negative = true;
                    int_part = -1 * int_part;
                )?

                $(dec_part = $float;)?
                $(suffix = LiteralSuffix::$suffix;)?

                NumLiteral {
                    negative,
                    int_part,
                    dec_part,
                    suffix,
                    float,
                }
            }
        };
    }

    #[test]
    fn parentheses() -> InterpreTestResult {
        let (input1, output1) = (
            "(())".chars().collect(),
            [Token::LParen, Token::LParen, Token::RParen, Token::RParen],
        );
        let (input2, output2) = (
            "((())".chars().collect(),
            [
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
            [
                Token::LParen,
                Token::from(NumLiteral::new_int(1243, false)),
                Token::RParen,
            ],
        );
        let (input2, output2) = (
            "(-124.3)".chars().collect(),
            [
                Token::LParen,
                Token::from(NumLiteral::new_float(124, 3, true)),
                Token::RParen,
            ],
        );
        let (input3, output3) = (
            "(-124f)".chars().collect(),
            [
                Token::LParen,
                Token::from(NumLiteral::new_int_with_suffix(124, true, 'f')),
                Token::RParen,
            ],
        );

        assert_eq!(tokenize(input1)?, output1);
        assert_eq!(tokenize(input2)?, output2);
        assert_eq!(tokenize(input3)?, output3);

        Ok(())
    }

    #[test]
    fn math_test() -> InterpreTestResult {
        let (input1, output1) = (
            // The lexer does no syntax or type analysis so this is fine
            "(+  -15.23f 1243u )".chars().collect(),
            [
                Token::LParen,
                Token::Plus,
                Token::from(NumLiteral::new_float_with_suffix(15, 23, true, 'f')),
                Token::from(NumLiteral::new_int_with_suffix(1243, false, 'u')),
                Token::RParen,
            ],
        );
        let (input2, output2) = (
            "(- -124c (/ 0.3 123u))".chars().collect(),
            [
                Token::LParen,
                Token::Minus,
                Token::from(NumLiteral::new_int_with_suffix(124, true, 'c')),
                Token::LParen,
                Token::Div,
                Token::from(NumLiteral::new_float(0, 3, false)),
                Token::from(NumLiteral::new_int_with_suffix(123, false, 'u')),
                Token::RParen,
                Token::RParen,
            ],
        );
        let (input3, output3) = (
            "(* (/ 2 9) -124f)".chars().collect(),
            [
                Token::LParen,
                Token::Mult,
                Token::LParen,
                Token::Div,
                Token::from(NumLiteral::new_int(2, false)),
                Token::from(NumLiteral::new_int(9, false)),
                Token::RParen,
                Token::from(NumLiteral::new_int_with_suffix(124, true, 'f')),
                Token::RParen,
            ],
        );

        assert_eq!(tokenize(input1)?, output1);
        assert_eq!(tokenize(input2)?, output2);
        assert_eq!(tokenize(input3)?, output3);

        Ok(())
    }
}
