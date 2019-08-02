extern crate pest;

use pest::error::Error;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct WaveguideParser;

pub type ParseResult<'a> = Pairs<'a, Rule>;
pub type ParseError = Error<Rule>;

pub fn parse(text: &str) -> Result<ParseResult, ParseError> {
    WaveguideParser::parse(Rule::root, text)
}

// We have to put this here because pest does not allow us to export the auto
// generated Rule enum.
pub mod convert {
    use super::*;
    use crate::vague::*;

    // Creates a variable, returns its id.
    fn parse_named_function_parameter(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> EntityId {
        let variable = VariableEntity::new();
        let mut name = Option::None;
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => (), // TODO: Use data type.
                Rule::identifier => name = Option::Some(part.as_str()),
                _ => unreachable!(),
            }
        }
        let id = program.adopt_entity(Entity::Variable(variable));
        program.define_symbol(func_scope, name.unwrap(), id);
        id
    }

    fn add_function_inputs(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            func.add_input(parse_named_function_parameter(program, func_scope, child));
        }
    }

    fn add_function_outputs(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            func.add_output(parse_named_function_parameter(program, func_scope, child));
        }
    }

    fn add_function_output(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        let variable = VariableEntity::new();
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => (), // TODO: Use data type.
                _ => unreachable!(),
            }
        }
        let var_id = program.adopt_entity(Entity::Variable(variable));
        program.define_symbol(func_scope, "!return_value", var_id);
        func.add_output(var_id);
    }

    fn convert_function_signature(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_inputs => add_function_inputs(program, func, func_scope, child),
                Rule::function_outputs => add_function_outputs(program, func, func_scope, child),
                Rule::single_function_output => {
                    add_function_output(program, func, func_scope, child)
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_returnable_code_block(
        program: &mut Program,
        scope: ScopeId,
        return_var: Option<EntityId>,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::statement => convert_statement(program, scope, child),
                Rule::expr => unimplemented!(),
                _ => unreachable!(),
            }
        }
    }

    fn convert_function_definition(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        let mut name = Option::None;
        let func_scope = program.create_child_scope(scope);
        let mut function = FunctionEntity::new(func_scope);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => name = Option::Some(child.as_str()),
                Rule::function_signature => {
                    convert_function_signature(program, &mut function, func_scope, child);
                }
                Rule::returnable_code_block => {
                    convert_returnable_code_block(
                        program,
                        func_scope,
                        function.get_single_output(),
                        child,
                    );
                }
                _ => unreachable!(),
            }
        }
        let function = program.adopt_entity(Entity::Function(function));
        // If name is None, there is a bug in the parser.
        program.define_symbol(scope, name.unwrap(), function);
    }

    fn convert_statement(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_definition => convert_function_definition(program, scope, child),
                Rule::code_block => unimplemented!(),
                Rule::create_variable_statement => unimplemented!(),
                Rule::assign_statement => unimplemented!(),
                Rule::raw_expr_statement => unimplemented!(),
                _ => unreachable!(),
            }
        }
    }

    pub fn convert_ast_to_vague(input: &mut ParseResult) -> Program {
        let root = input.next().unwrap();
        let mut program = Program::new();
        let scope = program.get_root_scope();

        for statement in root.into_inner() {
            match statement.as_rule() {
                Rule::EOI => continue,
                Rule::statement => convert_statement(&mut program, scope, statement),
                _ => unreachable!(),
            }
        }

        program
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid(text: &str) -> bool {
        return match parse(text) {
            Ok(pairs) => {
                println!("{:#?}", pairs);
                true
            }
            Err(error) => {
                println!("{:#?}", error);
                false
            }
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
