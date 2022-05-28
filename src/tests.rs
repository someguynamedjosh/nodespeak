#![cfg(test)]

use crate::{parser::parse_root, values::simplify::SimplificationContext, concrete::solidify};

#[test]
fn basic_parsing() {
    let file = r#"
    local thing = fn {
        output d: Array(Int, 5);
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
        // for (local, value) in assignments {
        //     if local.name == "thing" {
        //         println!("{:#?}", solidify(value));
        //     }
        // }
    } else {
        println!("{:#?}", result);
    }
    panic!();
}
