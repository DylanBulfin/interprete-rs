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

impl Rule {
    pub fn execute_rule(&self, tokens: &[Token]) -> ExecResult {
        match self {
            Rule::Prog => execute_prog(tokens),
            Rule::Expr => todo!(),
            Rule::ExprBody => todo!(),
            Rule::Val => todo!(),
            Rule::List => todo!(),
            Rule::ListBody => todo!(),
            Rule::FuncCall => todo!(),
            Rule::Args => todo!(),
        }
    }
}

fn execute_prog(tokens: &[Token]) -> ExecResult {
    execute_expr(tokens)
}

fn execute_expr(tokens: &[Token]) -> ExecResult {
    if tokens[0] == Token::LParen {
        let (child, cnt) = execute_expr_body(&tokens[1..])?;
        let node = Node::Rule(RuleNodeData::new(Rule::Expr, vec![Rc::new(child)]));

        if tokens[cnt + 1] == Token::RParen {
            Ok((node, cnt + 2))
        } else {
            Err(format!(
                "Expected ) while parsing expression, encountered {:?}",
                tokens[0]
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

fn execute_expr_body(tokens: &[Token]) -> ExecResult {
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

impl Tree {
}

//impl Default for Tree {
//    fn default() -> Self {
//        Self::new()
//    }
//}
