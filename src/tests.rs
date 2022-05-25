#![cfg(test)]

use crate::{parser::parse_root, values::simplify::SimplificationContext, concrete::solidify};

#[test]
fn basic_parsing() {
    let file = r#"
    local thing = fn {
        input a: Float;
        input b: Float;
        input c: Array(Float, 16);
        output d;

        d = add(add(a, b), c);
    };
"#;
    let result = parse_root(file);
    if let Ok((_, (_scope, statements))) = result {
        let mut ctx = SimplificationContext::new();
        let mut simplified = Vec::new();
        for statement in statements {
            statement.check_and_simplify(&mut ctx);
            simplified.push(statement);
        }
        let assignments = ctx.finish();
        println!("{:#?}", simplified);
        println!("{:#?}", assignments);
        for (local, value) in assignments {
            if local.name == "thing" {
                println!("{:#?}", solidify(value));
            }
        }
    } else {
        println!("{:#?}", result);
    }
    panic!();
}
