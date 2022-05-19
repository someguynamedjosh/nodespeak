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
    local add_anything = fn {
        input a: Array(Array(Float, 2), 128);
        output x = sub(add(a, 1), 0);
    };
    local thing = fn {
        input a: Int;
        output x = add_anything(cast(Float, a));
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
