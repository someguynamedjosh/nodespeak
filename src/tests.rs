#![cfg(test)]

use crate::{parser::parse_root, values::simplify::SimplificationContext};

#[test]
fn basic_parsing() {
    // local GLOBAL_BUFFER_SIZE = 512;
    // local GLOBAL_CHANNELS = 2;
    // local Signal = Array(Int, InSet(1, GLOBAL_BUFFER_SIZE), InSet(1, GLOBAL_CHANNELS));
    let file = r#"
    local test = fn {
        input a;
        output b = a;
    };
    test(1);
"#;
    let result = parse_root(file);
    if let Ok((_, (scope, statements))) = result {
        let mut ctx = SimplificationContext::new();
        let mut simplified = Vec::new();
        for statement in statements {
            simplified.push(statement.simplify(&mut ctx));
        }
        println!("{:#?}", simplified);
        println!("{:#?}", ctx.assignments());
    } else {
        println!("{:#?}", result);
    }
    panic!();
}
