use std::str::FromStr;

use proc_macro::{token_stream, TokenStream};
use syn;

#[proc_macro_attribute]
pub fn test_attr(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let mut code = item.to_string();

    code.push_str(
        format!(
            "pub fn attr_test() {{print!(\"{{}}\n{{}}\n\", \"{}\", \"{}\")}}",
            attr, item
        )
        .as_str(),
    );
    //TokenStream::from_str("fn plus_two(i: i32)->i32{i+2}").unwrap()
    TokenStream::from_str(&code).unwrap()
}

#[proc_macro_attribute]
pub fn reverse(_: TokenStream, item: TokenStream) -> TokenStream {
    let res: String = item
        .to_string()
        .chars()
        .rev()
        .map(|c| match c {
            '<' => '>',
            '>' => '<',
            '(' => ')',
            ')' => '(',
            ']' => '[',
            '[' => ']',
            '}' => '{',
            '{' => '}',
            d => d,
        })
        .collect();

    TokenStream::from_str(res.as_str()).unwrap()
}

#[proc_macro]
pub fn reverse_func(item: TokenStream) -> TokenStream {
    let res: String = item
        .to_string()
        .chars()
        .rev()
        .map(|c| match c {
            '<' => '>',
            '>' => '<',
            '(' => ')',
            ')' => '(',
            ']' => '[',
            '[' => ']',
            '}' => '{',
            '{' => '}',
            d => d,
        })
        .collect();

    TokenStream::from_str(res.as_str()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let result = add(2, 2);
        //assert_eq!(result, 4);
    }
}
