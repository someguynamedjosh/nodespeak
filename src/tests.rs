#![cfg(test)]

use crate::parser::parse_body;

#[test]
fn basic_parsing() {
    let file = r#"
    input blah;
    input thing: blah;
"#;
    let result = parse_body(file);
    println!("{:#?}", result);
    panic!();
}
