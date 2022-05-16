#![cfg(test)]

use crate::parser::parse_body;

#[test]
fn basic_parsing() {
    let file = r#"
    local GLOBAL_BUFFER_SIZE = 512;
    local GLOBAL_CHANNELS = 2;
    local Signal = Array(Int, InSet(1, GLOBAL_BUFFER_SIZE), InSet(1, GLOBAL_CHANNELS));
"#;
    let result = parse_body(file);
    println!("{:#?}", result);
    panic!();
}
