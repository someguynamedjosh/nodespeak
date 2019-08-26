use colored::*;
use pest::iterators::Pair;
use pest::RuleType;

pub struct FilePosition {
    start_pos: usize,
    start_line: usize,
    start_column: usize,
    end_pos: usize,
    end_line: usize,
    end_column: usize,
    source: Vec<String>,
}

impl FilePosition {
    pub fn from_pair<R: RuleType>(pair: &Pair<R>) -> FilePosition {
        let span = pair.as_span();
        let start = span.start_pos().line_col();
        let end = span.end_pos().line_col();
        FilePosition {
            start_pos: span.start(),
            start_line: start.0,
            start_column: start.1,
            end_pos: span.end(),
            end_line: end.0,
            end_column: end.1 - 1,
            source: span.lines().map(|item| item.to_owned()).collect(),
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
        if line_length + word_length > 80 {
            output.push('\n');
        }
        output.push_str(&word);
    }
    output
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

    pub fn format(&self, width: Option<usize>) -> String {
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
            let spacing = position.end_line.to_string().len();
            let spaces = &format!("{: ^1$}", "", spacing + 2);
            output.push_str(&format!("{:-^1$}\n", "", width).blue().to_string());
            output.push_str(&format!(
                "{}{}{}source:{}:{}\n",
                "|".blue().to_string(),
                spaces,
                "| ".blue().to_string(),
                position.start_line,
                position.start_column,
            ));
            output.push_str(&format!("{:-^1$}\n", "", width).blue().to_string());
            for (line, content) in position.source.iter().enumerate() {
                output.push_str(
                    &format!("| {: >1$} | ", (line + position.start_line), spacing)
                        .blue()
                        .to_string(),
                );
                let start_x = spacing + 5;
                let mut x = start_x;
                let mut column = 1;
                for ch in content.chars() {
                    if (line == 0 && column < position.start_column)
                        || (line == position.end_line - position.start_line
                            && column > position.end_column)
                    {
                        output.push(ch);
                    } else {
                        output.push_str(&match descriptor.ptype {
                            ProblemType::Error => ch.to_string().bright_red().to_string(),
                            ProblemType::Warning => ch.to_string().bright_yellow().to_string(),
                            ProblemType::Hint => ch.to_string().bright_cyan().to_string(),
                        });
                    }
                    x += 1;
                    column += 1;
                    if x > width {
                        output.push_str(&format!("\n|{}| ", spaces).blue().to_string());
                        x = start_x;
                    }
                }
            }
            output.push_str(&format!("{:-^1$}\n", "", width).blue().to_string());
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
        "Invalid Entity Name\nThere is no function, variable, or data type visible in this scope ",
        "with the specified name.",
    ),
    )])
}

pub fn return_from_root(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
        "Return Outside Function\nReturn statements can only be used inside of function ",
        "definitions. The code snippet below was understood to be a part of the root scope of the ",
        "file.",
    ),
    )])
}
