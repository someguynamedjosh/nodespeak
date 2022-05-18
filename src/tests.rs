#![cfg(test)]

use crate::{parser::parse_root, values::simplify::SimplificationContext};

#[test]
fn basic_parsing() {
    // local GLOBAL_BUFFER_SIZE = 512;
    // local GLOBAL_CHANNELS = 2;
    // local Signal = Array(Int, InSet(1, GLOBAL_BUFFER_SIZE), InSet(1,
    // GLOBAL_CHANNELS)); local test = fn {
    //     input a;
    //     output b = a;
    // };
    // test(add(1, 2));
    let file = r#"
    input size: InSet(1, 2);
    input size2: InSet(1, 2);
    add(Array(Int, size), Array(Int, size2));
"#;
    let result = parse_root(file);
    if let Ok((_, (_scope, statements))) = result {
        let mut ctx = SimplificationContext::new();
        let mut simplified = Vec::new();
        for statement in statements {
            statement.simplify(&mut ctx);
            simplified.push(statement);
        }
        let assignments = ctx.finish();
        println!("{:#?}", simplified);
        println!("{:#?}", assignments);
    } else {
        println!("{:#?}", result);
    }
    panic!();
}
