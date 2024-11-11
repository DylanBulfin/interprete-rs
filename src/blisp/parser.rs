use std::rc::Rc;

use crate::error::{InterpretError, InterpreteResult};

use super::{
    lexer::{NumLiteral, ReservedIdent, Token, Type},
    macros::rule_node_helper,
};

// usize is the number of tokens "consumed"
type ExecResult = InterpreteResult<(Node, usize)>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Rule {
    Prog,
    Expr,
    ExprBody,
    Val,
    List,
    ListBody,
    FuncCall,
    Args,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseToken {
    NumLiteral(NumLiteral),
    CharLiteral(u8),
    UnitLiteral,
    StringLiteral(String),
    Ident(String),
    Type(Type),
    Reserved(ReservedIdent),
}

impl From<NumLiteral> for ParseToken {
    fn from(value: NumLiteral) -> Self {
        Self::NumLiteral(value)
    }
}

impl From<u8> for ParseToken {
    fn from(value: u8) -> Self {
        Self::CharLiteral(value)
    }
}

impl From<String> for ParseToken {
    fn from(value: String) -> Self {
        Self::StringLiteral(value)
    }
}
impl From<&str> for ParseToken {
    fn from(value: &str) -> Self {
        Self::StringLiteral(value.to_string())
    }
}

impl From<Type> for ParseToken {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<ReservedIdent> for ParseToken {
    fn from(value: ReservedIdent) -> Self {
        Self::Reserved(value)
    }
}

impl TryFrom<Token> for ParseToken {
    type Error = InterpretError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::NumLiteral(n) => Ok(Self::NumLiteral(n)),
            Token::CharLiteral(c) => Ok(Self::CharLiteral(c)),
            Token::UnitLiteral => Ok(Self::UnitLiteral),
            Token::StringLiteral(s) => Ok(Self::StringLiteral(s)),
            Token::Ident(i) => Ok(Self::Ident(i)),
            Token::Type(t) => Ok(Self::Type(t)),
            Token::Reserved(r) => Ok(Self::Reserved(r)),
            t => Err(format!("{:?} is not a valid ParseToken", t).into()),
        }
    }
}

// Examples:
// `val_tokens_pat()` is an inline pattern that matches any token that can start `Val`. This is
//      used to check for `Val` in contexts where it is valid (e.g. `ExprBody`)
// `val_token_pat(terminals)` includes only the terminal states (e.g. the tokens). This is for
//      simplicity
macro_rules! val_tokens_pat {
    () => {
        Token::LBrack
            | Token::LParen
            | Token::Ident(_)
            | Token::Type(_)
            | Token::CharLiteral(_)
            | Token::StringLiteral(_)
            | Token::NumLiteral(_)
            | Token::UnitLiteral
    };
    (terminals) => {
        Token::Ident(_)
            | Token::Type(_)
            | Token::CharLiteral(_)
            | Token::StringLiteral(_)
            | Token::NumLiteral(_)
            | Token::UnitLiteral
    };
}
pub fn parse_prog(tokens: &[Token]) -> ExecResult {
    let (child, cnt) = parse_expr(tokens)?;
    let node = rule_node_helper!(Prog, child);

    if tokens
        .get(cnt)
        .ok_or("Unexpected end of token stream before EOF")?
        != &Token::EOF
    {
        Err(format!("Unexpected token where EOF was expected: {:?}", tokens[cnt]).into())
    } else {
        Ok((node, cnt))
    }
}

fn parse_expr(tokens: &[Token]) -> ExecResult {
    if tokens[0] == Token::LParen {
        let (child, cnt) = parse_expr_body(&tokens[1..])?;
        let node = rule_node_helper!(Expr, [child]);

        if tokens[cnt + 1] == Token::RParen {
            Ok((node, cnt + 2))
        } else {
            Err(format!(
                "Expected ) while parsing expression, encountered {:?}",
                tokens[cnt + 1]
            )
            .into())
        }
    } else {
        Err(format!(
            "Expected ( while parsing expression, encountered {:?}",
            tokens[0]
        )
        .into())
    }
}

fn parse_expr_body(tokens: &[Token]) -> ExecResult {
    match &tokens[0] {
        Token::Reserved(_) => {
            let (child, cnt) = parse_func_call(&tokens[0..])?;
            let node = rule_node_helper!(ExprBody, child);

            Ok((node, cnt))
        }
        val_tokens_pat!() => {
            // We have <Val> and need to process it
            let (child, cnt) = parse_val(&tokens[0..])?;
            let node = rule_node_helper!(ExprBody, child);

            Ok((node, cnt))
        }
        t => Err(format!(
            "Unexpected token encountered while parsing expression body: {:?}",
            t
        )
        .into()),
    }
}

fn parse_func_call(tokens: &[Token]) -> ExecResult {
    let func = tokens[0].assert_reserved()?;

    let (child, cnt) = parse_args(&tokens[1..])?;
    let node = rule_node_helper!(FuncCall, [Node::Leaf(ParseToken::from(*func)), child]);

    Ok((node, cnt + 1))
}

fn parse_args(tokens: &[Token]) -> ExecResult {
    match &tokens[0] {
        val_tokens_pat!() => {
            // We have <Val> and need to process it
            let (val, val_cnt) = parse_val(tokens)?;

            Ok(
                if tokens.get(val_cnt).ok_or::<InterpretError>(
                    "Unexpectedly reached end of input while parsing arguments".into(),
                )? == &Token::RParen
                {
                    (rule_node_helper!(Args, val), val_cnt)
                } else {
                    let (tail, tail_cnt) = parse_args(&tokens[1..])?;
                    (rule_node_helper!(Args, [val, tail]), val_cnt + tail_cnt)
                },
            )
        }
        t => Err(format!(
            "Unexpected token encountered while parsing arguments: {:?}",
            t
        )
        .into()),
    }
}

fn parse_val(tokens: &[Token]) -> ExecResult {
    match &tokens[0] {
        Token::LBrack => {
            let (child, cnt) = parse_list(tokens)?;
            let node = rule_node_helper!(Val, child);

            Ok((node, cnt))
        }
        Token::LParen => {
            let (child, cnt) = parse_expr(tokens)?;
            let node = rule_node_helper!(Val, child);

            Ok((node, cnt))
        }
        // terminals specifies that we want to leave out LBrack and LParen
        val_tokens_pat!(terminals) => {
            let child = Node::Leaf(tokens[0].clone().try_into()?);
            let node = rule_node_helper!(Val, child);

            Ok((node, 1))
        }
        _ => Err(format!("Unexpected token while parsing value: {:?}", tokens[0]).into()),
    }
}

fn parse_list(tokens: &[Token]) -> ExecResult {
    if tokens[0] == Token::LBrack {
        let (child, cnt) = parse_list_body(&tokens[1..])?;
        let node = rule_node_helper!(List, [child]);

        if tokens[cnt + 1] == Token::RBrack {
            Ok((node, cnt + 2))
        } else {
            Err(format!(
                "Expected ] while parsing list, encountered {:?}",
                tokens[cnt + 1]
            )
            .into())
        }
    } else {
        Err(format!("Expected [ while parsing list, encountered {:?}", tokens[0]).into())
    }
}

fn parse_list_body(tokens: &[Token]) -> ExecResult {
    match &tokens[0] {
        val_tokens_pat!() => {
            // We have <Val> and need to process it
            let (val, val_cnt) = parse_val(tokens)?;

            Ok(
                if tokens.get(val_cnt).ok_or::<InterpretError>(
                    "Unexpectedly reached end of input while trying to parse list".into(),
                )? == &Token::RBrack
                {
                    (rule_node_helper!(ListBody, val), val_cnt)
                } else {
                    let (tail, tail_cnt) = parse_list_body(&tokens[1..])?;
                    (rule_node_helper!(ListBody, [val, tail]), val_cnt + tail_cnt)
                },
            )
        }
        t => Err(format!(
            "Unexpected token encountered while parsing expression body: {:?}",
            t
        )
        .into()),
    }
}

// Want to create functions that "execute a rule" by gobbling tokens and return Nodes
pub struct Tree {
    base: Node,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Node {
    Leaf(ParseToken),
    Rule(RuleNodeData),
}

impl Node {
    //pub fn new_rule_node(rule: Rule, children: Vec<Node>) -> Self {
    //    Self::Rule(RuleNodeData {
    //        rule,
    //        children: children.into_iter().map(Rc::new).collect(),
    //    })
    //}
    pub fn is_val(&self) -> bool {
        matches!(
            self,
            Self::Rule(RuleNodeData {
                rule: Rule::Val,
                ..
            })
        )
    }
    pub fn is_func_call(&self) -> bool {
        matches!(
            self,
            Self::Rule(RuleNodeData {
                rule: Rule::FuncCall,
                ..
            })
        )
    }
}

impl From<ParseToken> for Node {
    fn from(value: ParseToken) -> Self {
        Node::Leaf(value)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RuleNodeData {
    rule: Rule,
    children: Vec<Rc<Node>>,
}

impl RuleNodeData {
    pub fn new(rule: Rule, children: Vec<Rc<Node>>) -> Self {
        Self { rule, children }
    }
}

impl Tree {}

//impl Default for Tree {
//    fn default() -> Self {
//        Self::new()
//    }
//}

#[cfg(test)]
mod tests {
    use crate::{
        blisp::{
            lexer::{tokenize, NumLiteral, Token, Type},
            macros::{assert_fails, assert_fails_parser, list_node_helper, val_node_helper},
            parser::parse_val,
        },
        error::InterpreTestResult,
    };

    use super::*;

    fn fails() {
        panic!("TestMessage");
    }

    assert_fails!(assert_fails_test => fails());
    assert_fails!(assert_fails_message_test => fails(); "TestMessage");

    // Creates a Prog node representing a program with a single Val in the main expression, e.g.
    // `(12)` or `(["ABC", 12])`
    //
    // You could create a parse tree for `(12)` with
    //      `nested_val_node_helper(Token::from(NumLiteral::new_int(12, false)))`
    // For `(["ABC", 12])` you could use `nested_val_node_helper([list_node])` where
    //      `list_node: Node::Rule(RuleNodeData { rule: Rule::List, .. })`
    macro_rules! nested_val_node_helper {
        ([$node:expr]) => {{
            let node = $node;
            let node = rule_node_helper!(Val, node);
            let node = rule_node_helper!(ExprBody, node);
            let node = rule_node_helper!(Expr, node);
            let node = rule_node_helper!(Prog, node);
            node
        }};
        ($tok:expr) => {{
            let node = Node::Leaf($tok);
            let node = rule_node_helper!(Val, node);
            let node = rule_node_helper!(ExprBody, node);
            let node = rule_node_helper!(Expr, node);
            let node = rule_node_helper!(Prog, node);
            node
        }};
    }

    macro_rules! do_parse_test {
        ($([$input:expr, $node:expr, $count:literal]),+) => {
            {
                $(
                    let input = $input.chars().collect();
                    let tokens = tokenize(input)?;
                    let node = $node;

                    assert_eq!(parse_prog(&tokens)?, (node, $count));
                )+

                Ok(())
            }
        };
    }

    //#[test]
    //fn parse_val_test() -> InterpreTestResult {
    //    let input1 = "(-1.24f)".chars().collect();
    //    let tokens1 = tokenize(input1)?;
    //    let node1 = val_node_helper!(Token::from(NumLiteral::new_float_with_suffix(
    //        1, 24, true, 'f'
    //    )));
    //
    //    let input2 = "(arstien)".chars().collect();
    //    let tokens2 = tokenize(input2)?;
    //    let node2 = val_node_helper!(Token::Ident("arstien".to_string()));
    //
    //    let input3 = "(uint)".chars().collect();
    //    let tokens3 = tokenize(input3)?;
    //    let node3 = val_node_helper!(Token::Type(Type::UInt));
    //
    //    assert_eq!(parse_prog(&tokens1)?, (node1, 3));
    //    assert_eq!(parse_prog(&tokens2)?, (node2, 3));
    //    assert_eq!(parse_prog(&tokens3)?, (node3, 3));
    //
    //    Ok(())
    //}

    #[test]
    fn parse_literal_val_test() -> InterpreTestResult {
        do_parse_test!(
            [
                "(-1.24f)",
                nested_val_node_helper!(ParseToken::from(NumLiteral::new_float_with_suffix(
                    1, 24, true, 'f'
                ))),
                3
            ],
            [
                "(arstien)",
                nested_val_node_helper!(ParseToken::Ident("arstien".to_string())),
                3
            ],
            [
                "(uint)",
                nested_val_node_helper!(ParseToken::Type(Type::UInt)),
                3
            ],
            [
                "('a')",
                nested_val_node_helper!(ParseToken::CharLiteral(b'a')),
                3
            ],
            [
                "(\"teststr\")",
                nested_val_node_helper!(ParseToken::StringLiteral("teststr".to_string())),
                3
            ],
            ["(())", nested_val_node_helper!(ParseToken::UnitLiteral), 3]
        )
    }

    #[test]
    fn parse_list_test() -> InterpreTestResult {
        let node1 = list_node_helper!(
            val_node_helper!(ParseToken::from(NumLiteral::new_float(12, 4, false))),
            val_node_helper!(ParseToken::CharLiteral(b'c')),
            val_node_helper!(ParseToken::from("ABCD".to_string()))
        );

        do_parse_test!(["([12.4 'c' \"ABCD\"])", nested_val_node_helper!([node1]), 7])
    }

    assert_fails_parser!(test_test, "(\"ABC\" 12)");
}
