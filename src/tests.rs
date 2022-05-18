#![cfg(test)]

use crate::{parser::parse_root, values::simplify::SimplificationContext, concrete::solidify};

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
    local thing = fn {
        input a: Int;
        output b = add(a, 1);
    };
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
        println!("{:#?}", solidify(assignments[0].1.ptr_clone()));
    } else {
        println!("{:#?}", result);
    }
    panic!();
}
