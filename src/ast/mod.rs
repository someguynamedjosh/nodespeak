use crate::problem::CompileProblem;
use crate::problem::FilePosition;

use pest::error::ErrorVariant;
use pest::Parser;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
struct WaveguideParser;

pub mod structure {
    pub use super::Rule as Rule;
    use pest::iterators::{Pair, Pairs};

    pub type Program<'a> = Pairs<'a, Rule>;
    pub type Node<'a> = Pair<'a, Rule>;

    pub fn rule_name(rule: &Rule) -> &'static str {
        match rule {
            Rule::WHITESPACE => "whitespace",
            Rule::EOI => "end of file",

            Rule::dec_int => "integer literal",
            Rule::hex_int => "hexadecimal literal",
            Rule::oct_int => "octal literal",
            Rule::legacy_oct_int => "c-style octal literal",
            Rule::bin_int => "binary literal",
            Rule::dec_digit => "digit",
            Rule::float => "float literal",
            Rule::int => "int literal",
            Rule::array_literal => "array literal",
            Rule::literal => "literal value",
            Rule::identifier => "identifier",

            Rule::expr_part_1 => "expression",
            Rule::expr_part_2 => "expression",
            Rule::expr_part => "expression",
            Rule::expr => "expression",
            Rule::negate => "unary negation",
            Rule::index_expr => "array access",
            Rule::operator => "binary operator",

            Rule::func_expr_input_list => "input list for function call",
            Rule::inline_output => "inline keyword",
            Rule::inline_var_dec => "inline variable declaration",
            Rule::func_expr_output => "output for function call",
            Rule::func_expr_output_list => "output list for function call",
            Rule::func_expr => "single-output function expression",

            Rule::named_data_type => "name of a data type",
            Rule::dynamic_data_type => "dynamic data type expression",
            Rule::basic_data_type => "data type",
            Rule::array_data_type => "array data type",
            Rule::data_type => "data type",

            Rule::named_function_parameter => "function parameter definition",
            Rule::function_inputs => "input list for function definition",
            Rule::function_outputs => "output list for function definition",
            Rule::single_function_output => "single output for function definition",
            Rule::function_signature => "signature for function definition",
            Rule::function_definition => "function definition",

            Rule::empty_variable => "uninitialized variable name",
            Rule::assigned_variable => "initialized variable declaration",
            Rule::create_variable => "variable declaration",
            Rule::create_variable_statement => "variable declaration statement",
            Rule::input_variable_statement => "input declaration statement",
            Rule::output_variable_statement => "output declaration statement",

            Rule::assign_array_access => "LHS assignment indexing",
            Rule::assign_expr => "LHS assignment expression",
            Rule::assign_statement => "assignment expression",

            Rule::code_block => "code block",
            Rule::returnable_code_block => "code block",
            Rule::return_statement => "return statement",

            Rule::assert_statement => "assert statement",

            Rule::raw_expr_statement => "expression as statement",
            Rule::statement => "statement",
            Rule::root => "program",
        }
    }
}

pub(self) mod problems {
    use crate::problem::{CompileProblem, FilePosition, ProblemDescriptor, ProblemType};

    pub fn bad_syntax(pos: FilePosition, message: String) -> CompileProblem {
        CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
            pos,
            ProblemType::Error,
            &format!("Bad Syntax\n{}", message),
        )])
    }
}

pub fn ingest(text: &str) -> Result<structure::Program, CompileProblem> {
    WaveguideParser::parse(Rule::root, text).map_err(|parse_err| {
        problems::bad_syntax(
            FilePosition::from_input_location(parse_err.location),
            match parse_err.variant {
                ErrorVariant::ParsingError {
                    positives,
                    negatives,
                } => format!(
                    "Expected {}... but found {}.",
                    {
                        positives
                            .iter()
                            .map(|rule| structure::rule_name(rule))
                            .collect::<Vec<&str>>()
                            .join(", ")
                    },
                    {
                        if negatives.len() == 0 {
                            "unknown syntax".to_owned()
                        } else {
                            negatives
                                .iter()
                                .map(|rule| structure::rule_name(rule))
                                .collect::<Vec<&str>>()
                                .join(", ")
                        }
                    }
                ),
                ErrorVariant::CustomError { message: _message } => {
                    unreachable!("Only parsing errors are encountered in the parser.")
                }
            },
        )
    })
}
