//! This holds some declarative macros that I wrote while working on BLisp. Some are for
//! testing, some are for convenience. All macros have test cases below, which can be used
//! as examples for syntax/usage. Since they are mostly not public members doctests don't
//! work so providing examples in the comments is subject to breakage.

//#![allow(unused_macros)]

macro_rules! import {
    (lexer) => {
        #[allow(unused_imports)]
        use $crate::blisp::lexer::*;
    };
    (parser) => {
        #[allow(unused_imports)]
        use $crate::blisp::parser::*;
    };
    (*) => {
        #[allow(unused_imports)]
        use $crate::blisp::lexer::*;
        #[allow(unused_imports)]
        use $crate::blisp::parser::*;
    };
}

// Assert that an expression panics, with an optional message.
macro_rules! assert_fails {
    ($testname:ident => $body:expr $(;$message:literal)?) => {
        #[test]
        #[should_panic$((expected = $message))?]
        fn $testname() { $body; }
    };
}
// Assert that an input string fails the lexing step (malformed literals, invalid
// characters, etc)
macro_rules! assert_fails_lexer {
    ($testname:ident, $input:literal $(;$message:literal)?) => {
        $crate::blisp::macros::import!(lexer);
        assert_fails!($testname => {
            if let Err(e) = tokenize($input.chars().collect()) {
                panic!("{}", e);
            }
        } $(;$message)?);
    };
}
// Assert that an input string fails the parsing step (mismatched parentheses, invalid
// tokens, etc)
macro_rules! assert_fails_parser {
    ($testname:ident, $input:literal $(;$message:literal)?) => {
        $crate::blisp::macros::import!(*);
        assert_fails!($testname => {
            if let Ok(tokens) = tokenize($input.chars().collect()) {
                if let Err(e) = parse_prog(tokens.as_slice()) {
                    panic!("{}", e);
                }
            }
        } $(;$message)?);
    };
}

// Allows more efficient rule node creation than a method since it avoids creating and
// immediately iterating/mapping a collection
macro_rules! rule_node_helper {
    ($rule:ident, $child:ident) => {
        {
            $crate::blisp::macros::import!(parser);
            Node::Rule(RuleNodeData::new(Rule::$rule, vec![std::rc::Rc::new($child)]))
        }
    };
    ($rule:ident, [$($child:expr),+]) => {
        {
            $crate::blisp::macros::import!(parser);
            Node::Rule(RuleNodeData::new(
                 Rule::$rule,
                 vec![$(std::rc::Rc::new($child),)+],
            ))
        }
    }
}

// Creates a Val node with Node::Leaf($tok) as the sole child
macro_rules! val_node_helper {
    ([$node:expr]) => {
        rule_node_helper!(Val, [$node])
    };
    ($tok:expr) => {
        rule_node_helper!(Val, [Node::from($tok)])
    };
}

// Creates a List node with the specified nodes as members. Input nodes must be of type
// Val (such as those created with above macro)
macro_rules! list_node_helper {
    [$($item:expr),+] => {{
        $crate::blisp::macros::import!(*);

        let mut vec: Vec<Node> = [$($item),+].to_vec();
        let mut node = rule_node_helper!(ListBody, [vec.pop().unwrap()]);

        while let Some(item) = vec.pop() {
            node = rule_node_helper!(ListBody, [item, node]);
        }

        node = rule_node_helper!(
            List,
            [node]
        );

        node
    }};
}

// Creates a FuncCall node with the specified function name and arguments
macro_rules! func_call_node_helper {
    ($func:ident, [$($arg:expr),+]) => {{
        $crate::blisp::macros::import!(*);

        let mut vec: Vec<Node> = [$($arg),+].to_vec();
        let mut node = rule_node_helper!(Args, [vec.pop().unwrap()]);

        while let Some(item) = vec.pop() {
            node = rule_node_helper!(Args, [item, node]);
        }

        let func_node = Node::Leaf(ParseToken::Reserved(ReservedIdent::$func));

        node = rule_node_helper!(
            FuncCall,
            [func_node, node]
        );

        node
    }};
}

// Create a full program node from a single base node. Needs a node of a type that can
// exist in the ExprBody context, e.g. either a Val node or FuncCall node
macro_rules! prog_node_helper {
    ($node:expr) => {
        if !node.is_val() || !node.is_func_call {
            panic!("prog_node_helper only supports Val or FuncCall nodes")
        }
        let node = $node;
        let node = rule_node_helper!(ExprBody, node);
        let node = rule_node_helper!(
            Expr,
            [Node::Leaf(Token::LParen), node, Node::Leaf(Token::RParen)]
        );
        let node = rule_node_helper!(Prog, node);
        node
    };
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        blisp::{lexer::Token, parser::Node},
        error::InterpreTestResult,
    };

    use super::*;

    // Test assert_fails
    assert_fails!(assert_fails_test1 => panic!("TestMessage"); "TestMessage");
    assert_fails!(assert_fails_test2=> panic!());

    // Test assert_fails_lexer
    assert_fails_lexer!(
        assert_fails_lexer_test1,
        "(\"arst)";
        "Unexpectedly reached end of input while parsing a string literal"
    );
    assert_fails_lexer!(
        assert_fails_lexer_test2,
        "\'12\'";
        "Did not find closing \' where expected while processing a char literal"
    );

    // Test assert_fails_parser
    assert_fails_parser!(
        assert_fails_parser_test1,
        "(+ 1 sub)";
        "Unexpected token encountered while parsing arguments: Reserved(Sub)"
    );
    assert_fails_parser!(
        assert_fails_parser_test2,
        "(['a' 12 \"ART\")";
        "Unexpected token encountered while parsing expression body: RParen"
    );

    #[test]
    fn rule_node_helper_test() -> InterpreTestResult {
        let node1 = rule_node_helper!(
            List,
            [
                Node::Leaf(ParseToken::UnitLiteral),
                Node::Leaf(ParseToken::CharLiteral(b'a'))
            ]
        );
        let node2 = rule_node_helper!(
            Args,
            [Node::Leaf(ParseToken::from("TESTSTR")), node1.clone()]
        );

        let exp1 = Node::Rule(RuleNodeData::new(
            Rule::List,
            vec![
                Rc::new(Node::Leaf(ParseToken::UnitLiteral)),
                Rc::new(Node::Leaf(ParseToken::from(b'a'))),
            ],
        ));
        let exp2 = Node::Rule(RuleNodeData::new(
            Rule::Args,
            vec![
                Rc::new(Node::Leaf(ParseToken::from("TESTSTR"))),
                Rc::new(Node::Rule(RuleNodeData::new(
                    Rule::List,
                    vec![
                        Rc::new(Node::Leaf(ParseToken::UnitLiteral)),
                        Rc::new(Node::Leaf(ParseToken::from(b'a'))),
                    ],
                ))),
            ],
        ));

        assert_eq!(node1, exp1);
        assert_eq!(node2, exp2);

        Ok(())
    }

    // Expected from expression `([12.4 'c' "ABCD"])`
    fn get_test_list_node() -> Node {
        Node::Rule(RuleNodeData::new(
            Rule::List,
            [Node::Rule(RuleNodeData::new(
                Rule::ListBody,
                [
                    rule_node_helper!(
                        Val,
                        [Node::from(ParseToken::from(NumLiteral::new_float(
                            12, 4, false
                        )))]
                    ),
                    Node::Rule(RuleNodeData::new(
                        Rule::ListBody,
                        [
                            rule_node_helper!(Val, [Node::from(ParseToken::CharLiteral(b'c'))]),
                            Node::Rule(RuleNodeData::new(
                                Rule::ListBody,
                                [rule_node_helper!(
                                    Val,
                                    [Node::from(ParseToken::StringLiteral("ABCD".to_string()))]
                                )]
                                .map(Rc::new)
                                .to_vec(),
                            )),
                        ]
                        .map(Rc::new)
                        .to_vec(),
                    )),
                ]
                .map(Rc::new)
                .to_vec(),
            ))]
            .map(Rc::new)
            .to_vec(),
        ))
    }

    // Expected from expression `(+ 13.5 'd' [12.4 'c' "ABCD"])`
    fn get_test_func_call_node() -> Node {
        Node::Rule(RuleNodeData::new(
            Rule::FuncCall,
            [
                Node::from(ParseToken::Reserved(ReservedIdent::Add)),
                Node::Rule(RuleNodeData::new(
                    Rule::Args,
                    [
                        rule_node_helper!(
                            Val,
                            [Node::from(ParseToken::from(NumLiteral::new_float(
                                13, 5, false
                            )))]
                        ),
                        Node::Rule(RuleNodeData::new(
                            Rule::Args,
                            [
                                rule_node_helper!(Val, [Node::from(ParseToken::CharLiteral(b'd'))]),
                                Node::Rule(RuleNodeData::new(
                                    Rule::Args,
                                    [rule_node_helper!(Val, [get_test_list_node()])]
                                        .map(Rc::new)
                                        .to_vec(),
                                )),
                            ]
                            .map(Rc::new)
                            .to_vec(),
                        )),
                    ]
                    .map(Rc::new)
                    .to_vec(),
                )),
            ]
            .map(Rc::new)
            .to_vec(),
        ))
    }

    #[test]
    fn validate_test_nodes() -> InterpreTestResult {
        // Since I went through the trouble of defining these test nodes I may as well
        // use them for direct testing

        let tokens1 = tokenize("([12.4 'c' \"ABCD\"])".chars().collect())?;
        let tokens2 = tokenize("(+ 13.5 'd' [12.4 'c' \"ABCD\"])".chars().collect())?;

        let node1 = parse_prog(tokens1.as_slice())?;
        let node2 = parse_prog(tokens2.as_slice())?;

        let exp1 = Node::Rule(RuleNodeData::new(
            Rule::Prog,
            vec![Rc::new(Node::Rule(RuleNodeData::new(
                Rule::Expr,
                vec![Rc::new(Node::Rule(RuleNodeData::new(
                    Rule::ExprBody,
                    vec![Rc::new(Node::Rule(RuleNodeData::new(
                        Rule::Val,
                        vec![Rc::new(get_test_list_node())],
                    )))],
                )))],
            )))],
        ));
        let exp2 = Node::Rule(RuleNodeData::new(
            Rule::Prog,
            vec![Rc::new(Node::Rule(RuleNodeData::new(
                Rule::Expr,
                vec![Rc::new(Node::Rule(RuleNodeData::new(
                    Rule::ExprBody,
                    vec![Rc::new(get_test_func_call_node())],
                )))],
            )))],
        ));

        assert_eq!(node1, (exp1, 7));
        assert_eq!(node2, (exp2, 10));

        Ok(())
    }

    #[test]
    fn val_node_helper_test() -> InterpreTestResult {
        let node1 = val_node_helper!(ParseToken::CharLiteral(b'a'));
        let node2 = val_node_helper!(ParseToken::from("ABC"));
        let node3 = val_node_helper!([get_test_list_node()]);

        let exp1 = rule_node_helper!(Val, [Node::from(ParseToken::CharLiteral(b'a'))]);
        let exp2 = rule_node_helper!(Val, [Node::from(ParseToken::from("ABC"))]);
        let exp3 = rule_node_helper!(Val, [get_test_list_node()]);

        assert_eq!(node1, exp1);
        assert_eq!(node2, exp2);
        assert_eq!(node3, exp3);

        Ok(())
    }

    #[test]
    fn list_node_helper_test() -> InterpreTestResult {
        let node = list_node_helper!(
            val_node_helper!(ParseToken::from(NumLiteral::new_float(12, 4, false))),
            val_node_helper!(ParseToken::CharLiteral(b'c')),
            val_node_helper!(ParseToken::from("ABCD".to_string()))
        );

        let exp = get_test_list_node();

        assert_eq!(node, exp);

        Ok(())
    }

    #[test]
    fn func_call_node_helper_test() -> InterpreTestResult {
        let node = func_call_node_helper!(
            Add,
            [
                val_node_helper!(ParseToken::from(NumLiteral::new_float(13, 5, false))),
                val_node_helper!(ParseToken::CharLiteral(b'd')),
                val_node_helper!([get_test_list_node()])
            ]
        );

        let exp = get_test_func_call_node();

        assert_eq!(node, exp);

        Ok(())
    }
}

// This macro exposes a list of macros to the rest of this crate. This is so that I can
// use my testing macros anywhere in my code without cluttering the namespace with macros
// that aren't useful outside the crate (e.g. by using `macro_export`)
macro_rules! crate_publish_macros {
    ($($macro:ident),+ $(,)?) => {
        $(
            #[allow(unused_imports)]
            pub(crate) use $macro;
        )+
    };
}

crate_publish_macros!(
    assert_fails,
    assert_fails_lexer,
    assert_fails_parser,
    rule_node_helper,
    val_node_helper,
    list_node_helper,
    import,
);
