//! This holds some declarative macros that I wrote while working on BLisp. Some are for
//! testing, some are for convenience.

// Assert that an expression panics, with an optional message.
//
// `use interprete_rs::blisp::macros::assert_fails;`
// `assert_fails!(assert_fails_test => panic!("TestMessage"); "TestMessage");`
// ```
macro_rules! assert_fails {
    ($testname:ident => $body:expr $(;$message:literal)?) => {
        #[test]
        #[should_panic$((expected = $message))?]
        fn $testname() { $body; }
    };
}
macro_rules! assert_fails_lexer {
    ($testname:ident, $input:literal $(;$message:literal)?) => {
        assert_fails!($testname => {
            $crate::blisp::lexer::tokenize($input.chars().collect()).unwrap();
        } $(;$message)?);
    };
}
macro_rules! assert_fails_parser {
    ($testname:ident, $input:literal $(;$message:literal)?) => {
        assert_fails!($testname => {
            if let Ok(tokens) = $crate::blisp::lexer::tokenize($input.chars().collect()) {
                $crate::blisp::parser::parse_prog(tokens.as_slice()).unwrap();
            }
        } $(;$message)?);
    };
}

/// This macro exposes a list of macros to the rest of this crate. This is so that I can
/// use my testing macros anywhere in my code without cluttering the namespace with macros
/// that aren't useful outside the crate (e.g. by using `macro_export`)
macro_rules! crate_publish_macros {
    ($($macro:ident),+) => {
        $(
            pub(crate) use $macro;
        )+
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    assert_fails!(assert_fails_test => panic!("TestMessage"); "TestMessage");
}

crate_publish_macros!(assert_fails, assert_fails_lexer, assert_fails_parser);
