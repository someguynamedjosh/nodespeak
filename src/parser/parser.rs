extern crate pest;

use pest::Parser;
use pest::iterators::Pairs;
use pest::error::Error;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct WaveguideParser;

pub type ParseResult<'a> = Pairs<'a, Rule>;
pub type ParseError = Error<Rule>;

pub fn parse(text: &str) -> Result<ParseResult, ParseError> {
    WaveguideParser::parse(Rule::root, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid(text: &str) -> bool {
        return match parse(text) {
            Ok(pairs) => { println!("{:#?}", pairs); true },
            Err(error) => { println!("{:#?}", error); false }
        };
    }

    #[test]
    fn basic_function_call() {
        assert!(is_valid("func();"));
        assert!(is_valid("test_function_12938 (  )   ;"));

        assert!(!is_valid("func(;"));
        assert!(!is_valid("func);"));
        assert!(!is_valid("12039821();"));
    }

    #[test]
    fn input_function_call() {
        assert!(is_valid("func(12);"));
        assert!(is_valid("func(12, 34  , 120);"));
    }

    #[test]
    fn output_function_call() {
        assert!(is_valid("func:();"));
        assert!(is_valid("func:(asdf);"));
        assert!(is_valid("func:(out1, out2  , out3);"));

        assert!(!is_valid("func:(123);"));
    }

    #[test]
    fn input_output_function_call() {
        assert!(is_valid("func():();"));
        assert!(is_valid("func(in1):(out1);"));
        assert!(is_valid("func(in1, in2):(out1, out2);"));

        assert!(!is_valid("func(in1, in2):(out1, 12);"));
    }

    #[test]
    fn lambda_function_call() {
        assert!(is_valid("func { };"));
        assert!(is_valid("func(in1):(out1) { };"));
        assert!(is_valid("func(in1):(out1) { func(in1):(out1); };"));
        assert!(is_valid("func(in1):(out1) { func(in1) };"));
        assert!(is_valid("func(in1):(out1) { } { } { };"));

        assert!(!is_valid("{ func(); };"));
    }

    #[test]
    fn adjective_function_call() {
        // According to grammar specification, all function calls must specify
        // at least one of: input list, output list, or code block with no
        // preceding adjectives. This makes the grammar unambiguous
        assert!(is_valid("func {} adj1;"));
        assert!(is_valid("func() adj1;"));
        assert!(is_valid("func:() adj1;"));

        // This is, so far, the only syntactically invalid type of function call
        // which does not have any alternate meaning. (E.G. func adj1; resolves
        // to a variable declaration, so it should be positively tested for in
        // another test.)
        assert!(!is_valid("func adj1 { };"));
    }

    #[test]
    fn variable_declaration() {
        assert!(is_valid("Int a;"));
        assert!(is_valid("Int a = 12;"));
        assert!(is_valid("Int a, b;"));
        assert!(is_valid("Int a = 12, b = 13;"));
    }

    #[test]
    fn variable_assignment() {
        assert!(is_valid("a;"));
        assert!(is_valid("a = 12;"));
    }

    #[test]
    fn array_declaration() {
        assert!(is_valid("[4]Int a;"));
        assert!(is_valid("[4][3]Int a;"));
        assert!(is_valid("[4]Int a = [1, 2, 3, 4];"));
    }

    #[test]
    fn arithmetic() {
        assert!(is_valid("a = 12 + 34;"));
        assert!(is_valid("a = 12 - 34;"));
        assert!(is_valid("a = 12 * 34;"));
        assert!(is_valid("a = 12 ** 34;"));
        assert!(is_valid("a = 12 / 34;"));
        assert!(is_valid("a = 12 // 34;"));
        assert!(is_valid("a = 12 % 34;"));
    }

    #[test]
    fn logic() {
        assert!(is_valid("a = 12 and 34;"));
        assert!(is_valid("a = 12 or 34;"));
        assert!(is_valid("a = 12 xor 34;"));
        assert!(is_valid("a = 12 nand 34;"));
        assert!(is_valid("a = 12 nor 34;"));
        assert!(is_valid("a = 12 xnor 34;"));
    }

    #[test]
    fn bitwise_logic() {
        assert!(is_valid("a = 12 band 34;"));
        assert!(is_valid("a = 12 bor 34;"));
        assert!(is_valid("a = 12 bxor 34;"));
        assert!(is_valid("a = 12 bnand 34;"));
        assert!(is_valid("a = 12 bnor 34;"));
        assert!(is_valid("a = 12 bxnor 34;"));
    }

    #[test]
    fn comparison() {
        assert!(is_valid("a = 12 == 34;"));
        assert!(is_valid("a = 12 != 34;"));
        assert!(is_valid("a = 12 >= 34;"));
        assert!(is_valid("a = 12 <= 34;"));
        assert!(is_valid("a = 12 > 34;"));
        assert!(is_valid("a = 12 < 34;"));
    }

    #[test]
    fn literals() {
        assert!(is_valid("a = 12;"));
        assert!(is_valid("a = 12.0;"));
        assert!(is_valid("a = 0.01;"));
        assert!(is_valid("a = .01;"));
        assert!(is_valid("a = -4;"));
        assert!(is_valid("a = -4.3e1;"));
        assert!(is_valid("a = -4.3e+1;"));
        assert!(is_valid("a = -4.3e-1;"));
        assert!(is_valid("a = -3e-1;"));
        assert!(is_valid("a = .1e-1;"));

        assert!(is_valid("a = -01_234567;"));
        assert!(is_valid("a = -0o1_234567;"));
        assert!(is_valid("a = -0x9_ABCDEFabcdef;"));
        assert!(is_valid("a = -0b0_1;"));
        assert!(is_valid("a = -0b0_1;"));

        assert!(!is_valid("a = 0b2"));
        assert!(!is_valid("a = 0o8"));
        assert!(!is_valid("a = 08"));
        assert!(!is_valid("a = 0xG"));
    }

    #[test]
    fn function_definition() {
        assert!(is_valid("fn main { }"));
        assert!(is_valid("fn main() { }"));
        assert!(is_valid("fn main:() { }"));
        assert!(is_valid("fn main:(Int a) { }"));
        assert!(is_valid("fn main:Int { }"));
    }
}