extern crate pest;

use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct TestParser;

pub fn test(text: &str) {
    let result = TestParser::parse(Rule::test, text);
    println!("{:#?}", result);
}
