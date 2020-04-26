use crate::problem::CompileProblem;
use crate::problem::FilePosition;

use pest::error::ErrorVariant;
use pest::Parser;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
struct NodespeakParser;

pub mod structure {
    pub use super::Rule;
    use pest::iterators::{Pair, Pairs};

    pub type Program<'a> = Pairs<'a, Rule>;
    pub type Node<'a> = Pair<'a, Rule>;

    pub fn rule_name(rule: &Rule) -> &'static str {
        match rule {
            Rule::WHITESPACE => "whitespace",
            Rule::COMMENT | Rule::block_comment | Rule::line_comment => "comment",
            Rule::EOI => "end of file",

            Rule::dec_int => "integer literal",
            Rule::hex_int => "hexadecimal literal",
            Rule::oct_int => "octal literal",
            Rule::legacy_oct_int => "c-style octal literal",
            Rule::bin_int => "binary literal",
            Rule::dec_digit => "digit",
            Rule::float => "float literal",
            Rule::int => "int literal",
            Rule::literal => "literal value",
            Rule::identifier => "identifier",

            Rule::vp_var => "variable",
            Rule::build_array => "array data",
            Rule::vpe_part_1 | Rule::vpe_part_2 | Rule::vpe_part | Rule::vpe => {
                "value-producing expression"
            }
            Rule::build_array_type => "array type",
            Rule::vp_index => "index expression",
            Rule::negate => "negate",
            Rule::operator => "binary operator",

            Rule::var_dec => "variable declaration",
            Rule::vc_identifier => "variable",
            Rule::vc_index => "index expression",
            Rule::vce => "value-consuming expression",

            Rule::func_call_input_list => "input list for function call",
            Rule::inline_output => "inline keyword",
            Rule::func_call_output => "output for function call",
            Rule::func_call_output_list => "output list for function call",
            Rule::func_call => "single-output function callession",

            Rule::named_function_parameter => "function parameter definition",
            Rule::function_inputs => "input list for function definition",
            Rule::function_outputs => "output list for function definition",
            Rule::single_function_output => "single output for function definition",
            Rule::function_signature => "signature for function definition",
            Rule::function_definition => "function definition",

            Rule::else_if_clause => "else if clause",
            Rule::else_clause => "else clause",
            Rule::if_statement => "if statement",
            Rule::for_loop_statement => "for loop",

            Rule::input_variable_statement => "input declaration statement",
            Rule::output_variable_statement => "output declaration statement",
            Rule::assign_statement => "assignment statement",
            Rule::raw_vpe_statement => "value-producing expression as a statement",
            Rule::raw_vce_statement => "value-consuming expression as a statement",
            Rule::return_statement => "return statement",
            Rule::assert_statement => "assert statement",
            Rule::statement => "statement",

            Rule::code_block => "code block",
            Rule::returnable_code_block => "code block",

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
    NodespeakParser::parse(Rule::root, text).map_err(|parse_err| {
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
