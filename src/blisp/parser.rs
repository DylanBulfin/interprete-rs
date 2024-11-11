use std::rc::Rc;

use crate::error::{InterpretError, InterpreteResult};

use super::lexer::Token;

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

// Examples:
// `rule_node_helper(Expr, child)` creates a new Node::Rule with rule Expr and `child` as the
//      only child
// `rule_node_helper(Args, [child1, child2])` creates a new Node::Rule with rule Args and 2
//      children: `child1, child2`
macro_rules! rule_node_helper {
    ($rule:ident, $child:ident) => {
        {
            Node::Rule(RuleNodeData::new(Rule::$rule, vec![Rc::new($child)]))
        }
    };
    ($rule:ident, [$($child:expr),+]) => {
        {
            Node::Rule(RuleNodeData {
                rule: Rule::$rule,
                children: vec![
                    $(Rc::new($child),)+
                ],
            })
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
        let node = rule_node_helper!(
            Expr,
            [Node::Leaf(Token::LParen), child, Node::Leaf(Token::RParen)]
        );

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
    let node = rule_node_helper!(FuncCall, [Node::Leaf(Token::Reserved(*func)), child]);

    Ok((node, cnt + 1))
}

fn parse_args(tokens: &[Token]) -> ExecResult {
    match &tokens[0] {
        val_tokens_pat!() => {
            // We have <Val> and need to process it
            let (val, val_cnt) = parse_val(tokens)?;

            Ok(
                if tokens.get(val_cnt).ok_or::<InterpretError>(
                    "Unexpectedly reached end of input while trying to parse arguments".into(),
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
            "Unexpected token encountered while parsing expression body: {:?}",
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
            let child = Node::Leaf(tokens[0].clone());
            let node = rule_node_helper!(Val, child);

            Ok((node, 1))
        }
        _ => Err(format!("Unexpected token while parsing value: {:?}", tokens[0]).into()),
    }
}

fn parse_list(tokens: &[Token]) -> ExecResult {
    if tokens[0] == Token::LBrack {
        let (child, cnt) = parse_list_body(&tokens[1..])?;
        let node = rule_node_helper!(
            List,
            [Node::Leaf(Token::LBrack), child, Node::Leaf(Token::RBrack)]
        );

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
    Leaf(Token),
    Rule(RuleNodeData),
}

impl Node {
    pub fn new_rule_node(rule: Rule, children: Vec<Node>) -> Self {
        Self::Rule(RuleNodeData {
            rule,
            children: children.into_iter().map(Rc::new).collect(),
        })
    }
}

impl From<Token> for Node {
    fn from(value: Token) -> Self {
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
            macros::{assert_fails, assert_fails_parser},
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
            let node = rule_node_helper!(
                Expr,
                [Node::Leaf(Token::LParen), node, Node::Leaf(Token::RParen)]
            );
            let node = rule_node_helper!(Prog, node);
            node
        }};
        ($tok:expr) => {{
            let node = Node::Leaf($tok);
            let node = rule_node_helper!(Val, node);
            let node = rule_node_helper!(ExprBody, node);
            let node = rule_node_helper!(
                Expr,
                [Node::Leaf(Token::LParen), node, Node::Leaf(Token::RParen)]
            );
            let node = rule_node_helper!(Prog, node);
            node
        }};
    }

    // Expects a list of Val nodes as input
    macro_rules! list_node_helper {
        [$($item:expr),+] => {{
            let mut vec: Vec<Node> = [$($item),+].into_iter().collect();
            let mut node = Node::new_rule_node(Rule::ListBody, vec![vec.pop().unwrap()]);

            for item in vec {
                node = Node::new_rule_node(Rule::ListBody, vec![item, node]);
            }

            node = Node::new_rule_node(
                Rule::List,
                vec![Node::from(Token::LBrack), node, Node::from(Token::RBrack)]
            );

            node
        }};
    }

    macro_rules! literal_val_node {
        ($tok:expr) => {
            Node::new_rule_node(Rule::Val, vec![Node::from($tok)])
        };
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
                nested_val_node_helper!(Token::from(NumLiteral::new_float_with_suffix(
                    1, 24, true, 'f'
                ))),
                3
            ],
            [
                "(arstien)",
                nested_val_node_helper!(Token::Ident("arstien".to_string())),
                3
            ],
            [
                "(uint)",
                nested_val_node_helper!(Token::Type(Type::UInt)),
                3
            ],
            [
                "('a')",
                nested_val_node_helper!(Token::CharLiteral(b'a')),
                3
            ],
            [
                "(\"teststr\")",
                nested_val_node_helper!(Token::StringLiteral("teststr".to_string())),
                3
            ],
            ["(())", nested_val_node_helper!(Token::UnitLiteral), 3]
        )
    }

    #[test]
    fn parse_list_test() -> InterpreTestResult {
        let list_node = Node::Rule(RuleNodeData {
            rule: Rule::List,
            children: [
                Node::from(Token::LBrack),
                Node::Rule(RuleNodeData {
                    rule: Rule::ListBody,
                    children: [
                        Node::new_rule_node(
                            Rule::Val,
                            vec![Node::from(Token::from(NumLiteral::new_float(12, 4, false)))],
                        ),
                        Node::Rule(RuleNodeData {
                            rule: Rule::ListBody,
                            children: [
                                Node::new_rule_node(
                                    Rule::Val,
                                    vec![Node::from(Token::CharLiteral(b'c'))],
                                ),
                                Node::Rule(RuleNodeData {
                                    rule: Rule::ListBody,
                                    children: [Node::new_rule_node(
                                        Rule::Val,
                                        vec![Node::from(Token::StringLiteral("ABCD".to_string()))],
                                    )]
                                    .map(Rc::new)
                                    .to_vec(),
                                }),
                            ]
                            .map(Rc::new)
                            .to_vec(),
                        }),
                    ]
                    .map(Rc::new)
                    .to_vec(),
                }),
                Node::from(Token::RBrack),
            ]
            .map(Rc::new)
            .to_vec(),
        });

        let new_list_node = list_node_helper!(
            literal_val_node!(Token::from(NumLiteral::new_float(12, 4, false))),
            literal_val_node!(Token::CharLiteral(b'c')),
            literal_val_node!(Token::from("ABCD".to_string()))
        );

        do_parse_test!(
            [
                "([12.4 'c' \"ABCD\"])",
                nested_val_node_helper!([new_list_node]),
                7
            ],
            [
                "([12.4 'c' \"ABCD\"])",
                nested_val_node_helper!([list_node]),
                7
            ]
        )
    }

    assert_fails_parser!(test_test, "(\"ABC\" 12)");
}
