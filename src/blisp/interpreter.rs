use crate::error::{InterpretError, InterpreteResult};

use super::{
    lexer::{ReservedIdent, Type},
    parser::{Node, ParseToken},
};

pub enum ValueData {
    Int(i64),
    UInt(u64),
    Float(f64),
    List(Vec<Value>),
    Unit,
    Char(u8),
    Bool(bool),
}

pub struct Value {
    ty: Type,
    val: ValueData,
}

impl Value {
    pub fn new(ty: Type, val: ValueData) {
        Self { ty, val }
    }
}

impl From<u8> for Value

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value {
            ty: Type::List(Box::new(Type::Char)),
            //val: ValueData::
        }
    }
}

impl TryFrom<ParseToken> for Value {
    type Error = InterpretError;

    fn try_from(value: ParseToken) -> Result<Self, Self::Error> {
        match value {
            ParseToken::NumLiteral(_) => todo!(),
            ParseToken::CharLiteral(_) => todo!(),
            ParseToken::UnitLiteral => todo!(),
            ParseToken::StringLiteral(s) => todo!(),
            t => Err("Expected a literal token".into()), //ParseToken::Ident(_) => todo!(),
                                                         //ParseToken::Type(_) => todo!(),
                                                         //ParseToken::Reserved(_) => todo!(),
        }
    }
}

pub struct Func {
    f: ReservedIdent,
    args: Vec<Value>,
}

pub fn eval_node(node: Node) -> InterpreteResult<Value> {
    unimplemented!()
}

fn eval_leaf_node(node: Node) -> InterpreteResult<Value> {
    if let Node::Leaf(tok) = node {
        match tok {
            ParseToken::NumLiteral(n) => todo!(),
            ParseToken::CharLiteral(_) => todo!(),
            ParseToken::UnitLiteral => todo!(),
            ParseToken::StringLiteral(_) => todo!(),
            ParseToken::Ident(_) => todo!(),
            ParseToken::Type(_) => todo!(),
            ParseToken::Reserved(_) => todo!(),
        }
    } else {
        Err(format!("Expected leaf node, got {:?}", node).into())
    }
}
