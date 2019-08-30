use colored::*;
use pest::iterators::Pair;
use pest::RuleType;
use std::iter::FromIterator;
use std::ops::Add;

#[derive(Clone, Debug)]
pub struct FilePosition {
    start_pos: usize,
    end_pos: usize,
}

impl FilePosition {
    pub fn from_pair<R: RuleType>(pair: &Pair<R>) -> FilePosition {
        let span = pair.as_span();
        FilePosition {
            start_pos: span.start(),
            end_pos: span.end(),
        }
    }
}

enum ProblemType {
    Error,
    Warning,
    Hint,
}

struct ProblemDescriptor {
    position: FilePosition,
    ptype: ProblemType,
    caption: String,
}

impl ProblemDescriptor {
    fn new(position: FilePosition, ptype: ProblemType, caption: &str) -> ProblemDescriptor {
        ProblemDescriptor {
            position,
            ptype,
            caption: caption.to_owned(),
        }
    }
}

pub struct CompileProblem {
    descriptors: Vec<ProblemDescriptor>,
}

const FALLBACK_ERROR_WIDTH: usize = 60;

fn wrap_text(input: &str, width: usize, offset: usize) -> String {
    let mut output = "".to_owned();
    let mut word = "".to_owned();
    let mut line_length = offset;

    for ch in input.chars() {
        if ch == ' ' {
            let word_length = word.len();
            if line_length + word_length > width {
                output.push('\n');
                line_length = 0;
            }
            line_length += word_length + 1;
            output.push_str(&word);
            output.push(' ');
            word = "".to_owned();
        } else if ch == '\n' {
            output.push_str(&word);
            word = "".to_owned();
            output.push('\n');
            line_length = 0;
        } else {
            word.push(ch)
        }
    }
    if word.len() > 0 {
        let word_length = word.len();
        if line_length + word_length > width {
            output.push('\n');
        }
        output.push_str(&word);
    }
    output
}

struct GrabResult {
    prefix: String,
    highlighted: String,
    suffix: String,

    line_number: usize,
    column_number: usize,
}

impl CompileProblem {
    fn new() -> CompileProblem {
        CompileProblem {
            descriptors: Vec::new(),
        }
    }

    fn from_descriptors(descriptors: Vec<ProblemDescriptor>) -> CompileProblem {
        CompileProblem { descriptors }
    }

    fn add_descriptor(&mut self, descriptor: ProblemDescriptor) {
        self.descriptors.push(descriptor)
    }

    fn grab_text<'a>(from: &'a str, at: &FilePosition) -> GrabResult {
        let start_char = at.start_pos;
        let end_char = at.end_pos;
        let mut result = GrabResult {
            prefix: String::new(),
            highlighted: String::new(),
            suffix: String::new(),

            line_number: 0,
            column_number: 0,
        };
        let mut line_buffer: Vec<char> = Vec::new();
        let mut line = 1;
        let mut column = 0;

        for (index, character) in from.chars().enumerate() {
            column += 1;
            if index < start_char {
                line_buffer.push(character);
            } else if index == start_char {
                result.prefix = String::from_iter(line_buffer.iter());
                result.highlighted.push(character);
                result.line_number = line;
                result.column_number = column;
            } else if index > start_char && index < end_char {
                result.highlighted.push(character);
            } else if character != '\n' {
                result.suffix.push(character);
            }
            if character == '\n' {
                line_buffer.clear();
                line += 1;
                column = 0;
                if index >= end_char {
                    return result;
                }
            }
        }

        result
    }

    // This whole thing is a mess but it doesn't need to run fast.
    pub fn format(&self, width: Option<usize>, source_code: &str) -> String {
        let width = width.unwrap_or(FALLBACK_ERROR_WIDTH);
        let mut output = "".to_owned();
        for descriptor in self.descriptors.iter() {
            output.push_str(&match descriptor.ptype {
                ProblemType::Error => "ERROR: ".bright_red().to_string(),
                ProblemType::Warning => "WARNING: ".bright_yellow().to_string(),
                ProblemType::Hint => "HINT: ".bright_cyan().to_string(),
            });
            output.push_str(&wrap_text(&descriptor.caption, width, 10));
            output.push_str("\n");

            let position = &descriptor.position;
            let grabbed = Self::grab_text(source_code, position);
            let spacing = grabbed.line_number.to_string().len();
            let spaces = &format!("{: ^1$}", "", spacing + 2);
            output.push_str(&format!("{:-^1$}\n", "", width).blue().to_string());
            output.push_str(&format!(
                "{}{}{}source:{}:{}\n",
                "|".blue().to_string(),
                spaces,
                "| ".blue().to_string(),
                grabbed.line_number,
                grabbed.column_number,
            ));
            output.push_str(&format!("{:-^1$}", "", width).blue().to_string());
            let highlight_start = grabbed.prefix.len();
            let highlight_end = highlight_start + grabbed.highlighted.len();
            let start_x = spacing + 5;
            let mut x = start_x;
            let mut line = grabbed.line_number;
            for (index, ch) in grabbed
                .prefix
                .add(&grabbed.highlighted.add(&grabbed.suffix))
                .chars()
                .enumerate()
            {
                if ch == '\n' || index == 0 {
                    output.push_str(
                        &format!("\n| {: >1$} | ", (line), spacing)
                            .blue()
                            .to_string(),
                    );
                    x = start_x;
                    line += 1;
                }
                if ch != '\n' {
                    if index < highlight_start || index >= highlight_end {
                        output.push(ch);
                    } else {
                        output.push_str(&match descriptor.ptype {
                            ProblemType::Error => ch.to_string().bright_red().to_string(),
                            ProblemType::Warning => ch.to_string().bright_yellow().to_string(),
                            ProblemType::Hint => ch.to_string().bright_cyan().to_string(),
                        });
                    }
                    x += 1;
                    if x >= width {
                        output.push_str(&format!("\n|{}| ", spaces).blue().to_string());
                        x = start_x;
                    }
                }
            }
            output.push_str(&format!("\n{:-^1$}\n", "", width).blue().to_string());
        }
        output
    }
}

use ProblemType::Error;
use ProblemType::Hint;
use ProblemType::Warning;

pub fn no_entity_with_name(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Invalid Entity Name\nThere is no function, variable, or data type visible in this ",
            "scope with the specified name.",
        ),
    )])
}

pub fn return_from_root(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Return Outside Function\nReturn statements can only be used inside of function ",
            "definitions. The code snippet below was understood to be a part of the root scope ",
            "of the file.",
        ),
    )])
}

pub fn extra_return_value(
    statement_pos: FilePosition,
    extra_pos: FilePosition,
    expected_num_values: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            statement_pos,
            Error,
            &format!(
                concat!(
                    "Too Many Return Values\nThe highlighted return statement specifies too many ",
                    "values. The function it is a part of only has {} output values.",
                ),
                expected_num_values
            ),
        ),
        ProblemDescriptor::new(
            extra_pos,
            Hint,
            concat!("This value has no corresponding output."),
        ),
    ])
}

pub fn missing_return_values(
    statement_pos: FilePosition,
    expected_num_values: usize,
    actual_num_values: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        statement_pos,
        Error,
        &format!(
            concat!(
                "Not Enough Return Values\nReturn statements must either specify values for every ",
                "output or specify no outputs. The highlighted return statement specifies values ",
                "for {} outputs, but the function it is a part of has {} outputs.",
            ),
            actual_num_values, expected_num_values,
        ),
    )])
}

pub fn missing_inline_return(
    func_call_pos: FilePosition,
    output_list_pos: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            func_call_pos,
            Error,
            concat!(
                "Missing Inline Return\nThe position of the highlighted function call requires it ",
                "to have an inline output so that the output can be used in an expression or ",
                "statement. However, there is no inline output argument in the output list.",
            ),
        ),
        ProblemDescriptor::new(
            output_list_pos,
            Hint,
            concat!(
                "Try replacing one of the output arguments with the keyword 'inline'. If the ",
                "function being called only has one output, you can also delete the output list ",
                "entirely to automatically make the only output inline."
            ),
        ),
    ])
}

pub fn hint_encountered_while_parsing(
    context_description: &str,
    assignment_pos: FilePosition,
    error: &mut CompileProblem,
) {
    error.add_descriptor(ProblemDescriptor::new(
        assignment_pos,
        Hint,
        &format!("Error encountered while parsing {}:", context_description),
    ));
}
