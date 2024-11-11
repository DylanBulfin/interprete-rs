use crate::error::{InterpretError, InterpreteResult};

use super::{
    lexer::{NumLiteral, ReservedIdent, Type},
    parser::{Node, ParseToken},
};

/// Holds the runtime type of the value. Number means it can be `uint, int, float` when
/// needed. NegNumber means it can be `int, float` when needed.
pub enum AbstractType {
    ConcreteType(Type),
    Number,
    NegNumber,
}

impl From<Type> for AbstractType {
    fn from(value: Type) -> Self {
        Self::ConcreteType(value)
    }
}

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
    ty: AbstractType,
    val: ValueData,
}

impl Value {
    pub fn new(ty: AbstractType, val: ValueData) -> Self {
        Self { ty, val }
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self {
            ty: Type::Char.into(),
            val: ValueData::Char(value),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value {
            ty: Type::List(Box::new(Type::Char)).into(),
            val: ValueData::List(value.as_bytes().iter().copied().map(Value::from).collect()),
        }
    }
}

impl TryFrom<NumLiteral> for Value {
    type Error = InterpretError;

    fn try_from(value: NumLiteral) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<ParseToken> for Value {
    type Error = InterpretError;

    fn try_from(value: ParseToken) -> Result<Self, Self::Error> {
        match value {
            ParseToken::NumLiteral(n) => Self::try_from(n),
            ParseToken::CharLiteral(c) => Ok(c.into()),
            ParseToken::UnitLiteral => Ok(Value::new(Type::Unit.into(), ValueData::Unit)),
            ParseToken::StringLiteral(s) => Ok(s.into()),
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
