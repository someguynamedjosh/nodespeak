#![cfg(test)]

use crate::{parser::parse_body, values::simplify::SimplificationContext};

#[test]
fn basic_parsing() {
    let file = r#"
    local GLOBAL_BUFFER_SIZE = 512;
    local GLOBAL_CHANNELS = 2;
    local Signal = Array(Int, InSet(1, GLOBAL_BUFFER_SIZE), InSet(1, GLOBAL_CHANNELS));
"#;
    let result = parse_body(file);
    if let Ok((_, (scope, statements))) = result {
        let mut ctx = SimplificationContext::new();
        let mut simplified = Vec::new();
        for statement in statements {
            simplified.push(statement.simplify(&mut ctx));
        }
        println!("{:#?}", simplified);
        println!("{:#?}", ctx.assignments());
    }
    panic!();
}
