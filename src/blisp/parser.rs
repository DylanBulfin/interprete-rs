use std::rc::Rc;

use crate::error::InterpreteResult;

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

macro_rules! rule_node_helper {
    ($rule:ident, $child:ident) => {
        {
            Node::Rule(RuleNodeData::new(Rule::$rule, vec![Rc::new($child)]))
        }
    };
    ($rule:ident, [$($child:expr),+]) => {
        {
            Node::Rule(RuleNodeData {
                rule: Rule::Expr,
                children: vec![
                    $(Rc::new($child),)+
                ],
            })
        }
    }
}
impl Rule {
    //pub fn execute_rule(&self, tokens: &[Token]) -> ExecResult {
    //    match self {
    //        Rule::Prog => execute_prog(tokens),
    //        Rule::Expr => todo!(),
    //        Rule::ExprBody => todo!(),
    //        Rule::Val => todo!(),
    //        Rule::List => todo!(),
    //        Rule::ListBody => todo!(),
    //        Rule::FuncCall => todo!(),
    //        Rule::Args => todo!(),
    //    }
    //}
}

pub fn parse_prog(tokens: &[Token]) -> ExecResult {
    parse_expr(tokens)
}

fn parse_expr(tokens: &[Token]) -> ExecResult {
    if tokens[0] == Token::LParen {
        let (child, cnt) = parse_expr_body(&tokens[1..])?;
        //let node = Node::Rule(RuleNodeData {
        //    rule: Rule::Expr,
        //    children: vec![
        //        Rc::new(Node::Leaf(Token::LParen)),
        //        Rc::new(child),
        //        Rc::new(Node::Leaf(Token::RParen)),
        //    ],
        //});
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
    let mut curr_index = 0;

    match &tokens[0] {
        Token::Reserved(_) => {
            let (child, cnt) = parse_func_call(&tokens[0..])?;
            //let node = Node::Rule(RuleNodeData::new(Rule::ExprBody, vec![Rc::new(child)]));
            let node = rule_node_helper!(ExprBody, child);

            Ok((node, cnt))
        }
        Token::LBrack
        | Token::LParen
        | Token::Ident(_)
        | Token::Type(_)
        | Token::CharLiteral(_)
        | Token::StringLiteral(_)
        | Token::NumLiteral(_)
        | Token::UnitLiteral => {
            // We have <Val> and need to process it
            unimplemented!()
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

    //let node = Node::Rule(RuleNodeData::new(Rule::FuncCall, vec![Rc::new(child)]));
    let node = rule_node_helper!(FuncCall, [Node::Leaf(Token::Reserved(*func)), child]);

    Ok((node, cnt + 1))
}

fn parse_args(tokens: &[Token]) -> ExecResult {
    match tokens
        .get(1)
        .ok_or("Unexpectedly reached end of input while trying to parse arguments")?
    {
        &Token::RParen => {
            // Reached end of processing
        }
        &Token::LBrack
        | &Token::LParen
        | &Token::Ident(_)
        | &Token::Type(_)
        | &Token::CharLiteral(_)
        | &Token::StringLiteral(_)
        | &Token::NumLiteral(_)
        | &Token::UnitLiteral => {
            // We have <Val> and need to process it
            let (val, val_cnt) = parse_val(tokens)?;
            let (tail, tail_cnt) = parse_args(tokens)?;

            let node = rule_node_helper!(Args, [val, tail]);

            Ok((node, val_cnt + tail_cnt))
        }
        t => Err(format!(
            "Unexpected token encountered while parsing expression body: {:?}",
            t
        )
        .into()),
    }
}

fn parse_val(tokens: &[Tokens]) -> ExecResult {
    unimplemented!()
}

// Want to create functions that "execute a rule" by gobbling tokens and return Nodes

pub struct Tree {
    base: Node,
}

pub enum Node {
    Leaf(Token),
    Rule(RuleNodeData),
}

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
