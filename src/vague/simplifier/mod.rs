use crate::problem::CompileProblem;
use super::structure::{DataType, Expression, KnownData, Program, ScopeId, VariableId};
use std::collections::HashMap;

mod expressions;
mod foundation;
mod helpers;
mod statements;
pub(self) mod util;
pub(self) mod problems;

pub fn simplify(program: &mut Program) -> Result<(), CompileProblem> {
    let entry_point = program.get_entry_point();
    let mut simplifier = ScopeSimplifier::new(program);
    simplifier.simplify_scope(entry_point)?;
    Result::Ok(())
}

pub(self) struct ScopeSimplifier<'a> {
    program: &'a mut Program,
    conversion_table: HashMap<VariableId, VariableId>,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be simplified to
    // different values depending on the types used for the inputs and outputs
    // of the function.
    conversion_stack: Vec<HashMap<VariableId, VariableId>>,
}

pub(self) enum Content {
    /// A simpler or simplified version of the expression was found.
    Modified(Expression),
    /// The entire value of the expression has a determinate value.
    Interpreted(KnownData),
}

pub(self) struct SimplifiedExpression {
    pub content: Content,
    pub data_type: DataType,
}
