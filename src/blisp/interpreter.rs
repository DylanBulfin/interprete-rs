use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::format,
};

use crate::{
    blisp::macros::leaf_node_pattern,
    error::{InterpreTestResult, InterpretError, InterpreteResult},
};

use super::{
    lexer::{LiteralSuffix, NumLiteral, ReservedIdent, Type},
    macros::rule_node_pattern,
    parser::{Node, ParseToken, ParseTree, Rule, RuleNodeData},
};

/// Contains variable dictionary
pub struct State {
    vars: HashMap<String, Option<Value>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Get the value of the variable with specified identifier. Returns an Err if the
    pub fn get_var(&self, ident: &str) -> InterpreteResult<&Value> {
        self.vars
            .get(ident)
            .map(Option::as_ref)
            .ok_or(format!("Variable has not been initialized at all: {}", ident).into())
            .and_then(|o| o.ok_or("Variable has been initialized but not set".into()))
    }

    pub fn create_var(&mut self, ident: String, val: Option<Value>) -> InterpreteResult<()> {
        match self.vars.entry(ident) {
            Entry::Vacant(e) => {
                e.insert(val);
                Ok(())
            }
            Entry::Occupied(e) => {
                Err(format!("Already have a variable called: {}", e.key()).into())
            }
        }
    }

    pub fn set_var(&mut self, ident: String, val: Value) -> InterpreteResult<()> {
        match self.vars.entry(ident) {
            Entry::Occupied(mut e) => {
                e.insert(Some(val));
                Ok(())
            }
            Entry::Vacant(e) => {
                Err(format!("No variable exists with identifier {}", e.key()).into())
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the runtime type of the value. Number means it can be `uint, int, float` when
/// needed. NegNumber means it can be `int, float` when needed.
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum ValueData {
    Int(i64),
    UInt(u64),
    Float(f64),
    List(Vec<Value>),
    Unit,
    Char(u8),
    Bool(bool),
    // Abstract types below
    Number(u64),
    NegNumber(i64),
}

#[derive(Debug, PartialEq, Clone)]
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

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            ty: Type::Float.into(),
            val: ValueData::Float(value),
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

impl From<()> for Value {
    fn from(value: ()) -> Self {
        Value {
            ty: Type::Unit.into(),
            val: ValueData::Unit,
        }
    }
}

impl TryFrom<NumLiteral> for Value {
    type Error = InterpretError;

    fn try_from(value: NumLiteral) -> Result<Self, Self::Error> {
        match value {
            NumLiteral {
                suffix: LiteralSuffix::None,
                negative,
                float,
                int_part,
                ..
            } => {
                if float {
                    Ok(value.to_f64_checked()?.into())
                } else if negative {
                    Ok(Value::new(
                        AbstractType::NegNumber,
                        ValueData::NegNumber(-(int_part as i64)),
                    ))
                } else {
                    Ok(Value::new(
                        AbstractType::Number,
                        ValueData::Number(int_part),
                    ))
                }
            }
            NumLiteral {
                suffix: LiteralSuffix::Char,
                negative,
                float,
                int_part,
                ..
            } => {
                if float || negative || int_part > 255 {
                    Err(
                        format!("Unable to process NumLiteral with Char suffix: {:?}", value)
                            .into(),
                    )
                } else {
                    Ok(Value::new(
                        Type::Char.into(),
                        ValueData::Char(int_part as u8),
                    ))
                }
            }
            NumLiteral {
                suffix: LiteralSuffix::Float,
                ..
            } => Ok(value.to_f64_checked()?.into()),
            NumLiteral {
                suffix: LiteralSuffix::Unsigned,
                negative,
                float,
                int_part,
                ..
            } => {
                if float || negative {
                    Err(format!(
                        "Unable to process NumLiteral with Unsigned suffix: {:?}",
                        value
                    )
                    .into())
                } else {
                    Ok(Value::new(Type::UInt.into(), ValueData::UInt(int_part)))
                }
            }
        }
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

pub enum ArgumentType {
    Value,
    Type,
    Ident,
}

//pub fn get_arg_types(func: ReservedIdent) -> Vec<AbstractType> {
//
//}

pub enum Argument {
    Value(Value),
    Type(Type),
    Ident(String),
}

pub struct Func {
    f: ReservedIdent,
    args: Vec<Argument>,
}

pub fn eval(node: Node) -> InterpreteResult<Value> {
    let mut state = State::new();

    eval_prog_node(node, &mut state)
}

pub fn eval_node(node: Node) -> InterpreteResult<Value> {
    unimplemented!()
}

//fn eval_rule_node(node: Node, state: &mut state) -> InterpreteResult<Value> {
//    if let Node::Rule(data) = node {
//        match data {
//            RuleNodeData { rule: Rule::Prog, children }
//        }
//    }
//}

fn eval_leaf_node(node: Node, state: &State) -> InterpreteResult<Value> {
    if let Node::Leaf(tok) = node {
        match tok {
            ParseToken::NumLiteral(n) => n.try_into(),
            ParseToken::CharLiteral(c) => Ok(c.into()),
            ParseToken::UnitLiteral => Ok(().into()),
            ParseToken::StringLiteral(s) => Ok(s.into()),
            ParseToken::Ident(i) => state.get_var(&i).cloned(),
            t => Err(format!("Expected literal or identifier, found {:?}", t).into()),
        }
    } else {
        Err(format!("Expected leaf node, got {:?}", node).into())
    }
}

fn eval_prog_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let Node::Rule(RuleNodeData {
        rule: Rule::Prog,
        mut children,
    }) = node
    {
        assert!(children.len() == 1);
        eval_expr_node(children.pop().unwrap(), state)
    } else {
        Err(format!("Expected Prog node, found: {:?}", node).into())
    }
}

fn eval_expr_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let Node::Rule(RuleNodeData {
        rule: Rule::Expr,
        mut children,
    }) = node
    {
        assert!(children.len() == 1);
        eval_expr_body_node(children.pop().unwrap(), state)
    } else {
        Err(format!("Expected Expr node, found: {:?}", node).into())
    }
}

fn eval_expr_body_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let Node::Rule(RuleNodeData {
        rule: Rule::ExprBody,
        mut children,
    }) = node
    {
        assert!(children.len() == 1);
        let node = children.pop().unwrap();

        match &node {
            Node::Rule(RuleNodeData {
                rule: Rule::Val, ..
            }) => eval_val_node(node, state),
            Node::Rule(RuleNodeData {
                rule: Rule::FuncCall,
                ..
            }) => eval_func_call_node(node, state),
            _ => Err(format!(
                "Encountered unexpected node when evaluating expression body: {:?}",
                node
            )
            .into()),
        }
    } else {
        Err(format!("Expected ExprBody node, found: {:?}", node).into())
    }
}

fn eval_val_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let rule_node_pattern!(Val;mut children) = node {
        assert!(children.len() == 1);

        match children.pop().unwrap() {
            leaf_node_pattern!(CharLiteral(c)) => Ok(c.into()),
            leaf_node_pattern!(StringLiteral(s)) => Ok(s.into()),
            leaf_node_pattern!(NumLiteral(n)) => n.try_into(),
            leaf_node_pattern!(UnitLiteral) => Ok(().into()),
            leaf_node_pattern!(Ident(i)) => state.get_var(&i).cloned(),
            rule_node_pattern!(List => node) => eval_list_node(node, state),
            rule_node_pattern!(Expr => node) => eval_expr_node(node, state),
            n => Err(format!("Encountered invalid node when evaluating Val: {:?}", n).into()),
        }
    } else {
        Err(format!("Expected Expr node, found: {:?}", node).into())
    }
}

fn eval_func_call_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let rule_node_pattern!(FuncCall; mut children) = node {
        assert!(children.len() == 2);

        match children.pop().unwrap() {
            leaf_node_pattern!(Reserved(rsv)) => {
                let func = rsv;
                let args = eval_args_node(children.pop().unwrap(), state, func);

                unimplemented!()
            }
            n => Err(format!("Expected function name, found {:?}", n).into()),
        }
    } else {
        Err(format!("Expected Expr node, found: {:?}", node).into())
    }
}

fn eval_args_node(
    node: Node,
    state: &mut State,
    func: ReservedIdent,
) -> InterpreteResult<Argument> {
    unimplemented!()
}

fn eval_list_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    if let Node::Rule(RuleNodeData {
        rule: Rule::List,
        mut children,
    }) = node
    {
        assert!(children.len() == 1);
        eval_list_body_node(children.pop().unwrap(), state)
    } else {
        Err(format!("Expected Expr node, found: {:?}", node).into())
    }
}

fn eval_list_body_node(node: Node, state: &mut State) -> InterpreteResult<Value> {
    unimplemented!()
}

#[cfg(test)]
mod tests {

    use crate::{
        blisp::{lexer::tokenize, macros::assert_fails, parser::parse_prog},
        error::InterpreTestResult,
    };

    use super::*;

    #[test]
    fn num_literal_conversion_test() -> InterpreTestResult {
        let num1 = NumLiteral::new_int_with_suffix(1, true, 'f');
        let num2 = NumLiteral::new_float(1, 5, false);
        let num3 = NumLiteral::new_int_with_suffix(48, false, 'c');
        let num4 = NumLiteral::new_int(48, false);

        assert_eq!(num1.to_f64_checked().unwrap(), -1f64);
        assert_eq!(num2.to_f64_checked().unwrap(), 1.5f64);
        assert_eq!(Value::try_from(num3)?, Value::from(b'0'));
        assert_eq!(
            Value::try_from(num4)?,
            Value::new(AbstractType::Number, ValueData::Number(48))
        );

        Ok(())
    }

    assert_fails!(
        num_literal_invalid_test1 =>
        Value::try_from(NumLiteral::new_int_with_suffix(1, true, 'c')).unwrap()
    );

    assert_fails!(
        num_literal_invalid_test2 =>
        Value::try_from(NumLiteral::new_int_with_suffix(1, true, 'u')).unwrap()
    );

    macro_rules! do_eval_test {
        ($([$input:expr, $value:expr]),+ $(,)?) => {
            $(
                {
                    let input = $input.chars().collect();
                    let tokens = tokenize(input)?;
                    let prog = parse_prog(tokens.as_slice())?;
                    let value = eval(prog.0)?;

                    assert_eq!(value, $value);
                }
            )+
        };
    }

    #[test]
    fn basic_val_test() -> InterpreTestResult {
        let input1 = "(-1.2)".chars().collect();
        let tokens1 = tokenize(input1)?;
        let prog1 = parse_prog(tokens1.as_slice())?;
        let value1 = eval(prog1.0)?;
        let exp1 = Value {
            ty: Type::Float.into(),
            val: ValueData::Float(-1.2),
        };

        assert_eq!(value1, exp1);

        do_eval_test!(
            ["(48c)", Value::from(b'0')],
            [
                "(123)",
                Value::new(AbstractType::Number, ValueData::Number(123))
            ],
            [
                "(-123)",
                Value::new(AbstractType::NegNumber, ValueData::NegNumber(-123))
            ],
        );

        Ok(())
    }
}
