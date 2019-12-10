use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::collections::HashMap;

mod data_type;
mod expressions;
mod foundation;
mod helpers;
pub(self) mod problems;
mod statements;
pub(self) mod util;

pub(self) use data_type::*;

pub fn ingest(program: &mut i::Program) -> Result<o::Program, CompileProblem> {
    let entry_point = program.get_entry_point();
    let inputs = program[entry_point].borrow_inputs().clone();
    let outputs = program[entry_point].borrow_outputs().clone();
    let mut simplifier = ScopeSimplifier::new(program);
    let new_scope = simplifier.simplify_scope(entry_point)?;
    let inputs: Vec<_> = inputs
        .into_iter()
        .map(|id| simplifier.convert(id))
        .collect();
    let outputs: Vec<_> = outputs
        .into_iter()
        .map(|id| simplifier.convert(id))
        .collect();
    drop(simplifier);
    for input in inputs {
        program[new_scope].add_input(input);
    }
    for output in outputs {
        program[new_scope].add_output(output);
    }
    program.set_entry_point(new_scope);
    Result::Ok(simplifier.target)
}

#[derive(Clone, Debug)]
struct SimplifierTable {
    pub conversions: HashMap<i::VariableId, o::VariableId>,
    pub replacements: HashMap<i::VariableId, o::Expression>,
}

impl SimplifierTable {
    pub(self) fn new() -> SimplifierTable {
        SimplifierTable {
            conversions: HashMap::new(),
            replacements: HashMap::new(),
        }
    }
}

pub(super) struct ScopeSimplifier<'a> {
    source: &'a mut i::Program,
    target: o::Program,
    current_scope: o::ScopeId,
    table: SimplifierTable,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be simplified to
    // different values depending on the types used for the inputs and outputs
    // of the function.
    stack: Vec<SimplifierTable>,
    temp_values: HashMap<i::VariableId, i::KnownData>,
}

pub(self) enum Content {
    /// A simpler or simplified version of the expression was found.
    Modified(o::Expression),
    /// The entire value of the expression has a determinate value.
    Interpreted(i::KnownData),
}

impl Content {
    pub(self) fn into_expression(self, position: FilePosition) -> o::Expression {
        match self {
            Content::Modified(expr) => expr,
            Content::Interpreted(data) => unimplemented!(),
        }
    }
}

pub(self) struct SimplifiedExpression {
    pub content: Content,
    pub data_type: DataType,
}
