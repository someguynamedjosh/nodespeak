use super::structure::{DataType, Expression, KnownData, Program, VariableId};
use crate::problem::{CompileProblem, FilePosition};
use std::collections::HashMap;

mod expressions;
mod foundation;
mod helpers;
pub(self) mod problems;
mod statements;
pub(self) mod util;

pub fn simplify(program: &mut Program) -> Result<(), CompileProblem> {
    let entry_point = program.get_entry_point();
    let mut simplifier = ScopeSimplifier::new(program);
    let new_scope = simplifier.simplify_scope(entry_point)?;
    program.set_entry_point(new_scope);
    Result::Ok(())
}

#[derive(Clone, Debug)]
struct SimplifierTable {
    pub conversions: HashMap<VariableId, VariableId>,
    pub replacements: HashMap<VariableId, Expression>,
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
    program: &'a mut Program,
    table: SimplifierTable,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be simplified to
    // different values depending on the types used for the inputs and outputs
    // of the function.
    stack: Vec<SimplifierTable>,
}

pub(self) enum Content {
    /// A simpler or simplified version of the expression was found.
    Modified(Expression),
    /// The entire value of the expression has a determinate value.
    Interpreted(KnownData),
}

impl Content {
    pub(self) fn into_expression(self, position: FilePosition) -> Expression {
        match self {
            Content::Modified(expr) => expr,
            Content::Interpreted(data) => Expression::Literal(data, position),
        }
    }
}

pub(self) struct SimplifiedExpression {
    pub content: Content,
    pub data_type: DataType,
}
