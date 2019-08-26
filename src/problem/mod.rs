use colored::*;
use pest::RuleType;
use pest::iterators::Pair;

pub struct FilePosition {
    start_pos: usize,
    start_line: usize,
    start_column: usize,
    end_pos: usize,
    end_line: usize,
    end_column: usize,
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
        }
    }
}

pub struct ProblemDescriptor {
    position: FilePosition,
    caption: String
}

impl ProblemDescriptor {
    fn new(position: FilePosition, caption: &str) -> ProblemDescriptor {
        ProblemDescriptor {
            position,
            caption: caption.to_owned()
        }
    }
}

pub struct CompileProblem {
    descriptors: Vec<ProblemDescriptor>
}

fn wrap_80_ch(input: &str) -> String {
    let mut output = "".to_owned();
    let mut word = "".to_owned();
    let mut line_length = 0;

    for ch in input.chars() {
        if ch == ' ' {
            let word_length = word.len();
            if line_length + word_length > 80 {
                output.push('\n');
                line_length = 0;
            }
            line_length += word_length + 1;
            output.push_str(&word);
            output.push(' ');
            word = "".to_owned();
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
            descriptors: Vec::new()
        }
    }

    fn add_descriptor(&mut self, descriptor: ProblemDescriptor) {
        self.descriptors.push(descriptor)
    }

    pub fn terminal_format(&self, source_code: &str) -> String {
        let mut output = "".to_owned();
        for descriptor in self.descriptors.iter() {
            output.push_str(&wrap_80_ch(&descriptor.caption));
            let position = &descriptor.position;
            output.push_str(&format!("\nAt: source:{}:{}", position.start_line, position.start_column));
        }
        output.red().to_string()
    }
}