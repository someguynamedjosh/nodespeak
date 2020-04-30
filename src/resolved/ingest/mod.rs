use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::collections::{HashMap, HashSet};

mod data_type;
mod foundation;
mod helpers;
pub(self) mod problems;
mod statements;
pub(self) mod util;
mod vcexpression;
mod vpexpression;

pub(self) use data_type::*;

pub fn ingest(program: &mut i::Program) -> Result<o::Program, CompileProblem> {
    let entry_point = program.get_entry_point();
    let inputs = program[entry_point].borrow_inputs().clone();
    let outputs = program[entry_point].borrow_outputs().clone();
    let mut resolver = ScopeSimplifier::new(program);
    let new_entry_point = resolver.resolve_scope(entry_point)?;
    let inputs: Vec<_> = inputs
        .into_iter()
        .map(|id| {
            resolver
                .get_var_info(id)
                .expect("undefined input, should be caught in vague phase.")
                .0
        })
        .collect();
    let outputs: Vec<_> = outputs
        .into_iter()
        .map(|id| {
            resolver
                .get_var_info(id)
                .expect("undefined input, should be caught in vague phase.")
                .0
        })
        .collect();
    let mut result = resolver.target;

    for input in inputs {
        if let Option::Some(var_id) = input {
            result.add_input(var_id);
        } else {
            panic!("TODO: Nice error, input not available at run time.")
        }
    }
    for output in outputs {
        if let Option::Some(var_id) = output {
            result.add_output(var_id);
        } else {
            panic!("TODO: Nice error, input not available at run time.")
        }
    }
    result.set_entry_point(new_entry_point);
    Result::Ok(result)
}

#[derive(Clone, Debug)]
struct SimplifierTable {
    pub variables: HashMap<i::VariableId, (Option<o::VariableId>, DataType)>,
    pub unresolved_auto_vars: HashSet<i::VariableId>,
}

impl SimplifierTable {
    pub(self) fn new() -> SimplifierTable {
        SimplifierTable {
            variables: HashMap::new(),
            unresolved_auto_vars: HashSet::new(),
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
    // multiple conversions. For example, type parameters can be resolved to
    // different values depending on the types used for the inputs and outputs
    // of the function.
    stack: Vec<SimplifierTable>,
    pub temp_values: HashMap<i::VariableId, i::KnownData>,
}

#[derive(Clone, Debug)]
pub(self) enum SimplifiedVPExpression {
    /// A simpler or resolved version of the expression was found.
    Modified(o::VPExpression, DataType),
    /// The entire value of the expression has a determinate value.
    Interpreted(i::KnownData, FilePosition, DataType),
}

impl SimplifiedVPExpression {
    pub(self) fn borrow_data_type(&self) -> &DataType {
        match self {
            Self::Modified(_, dtype) => dtype,
            Self::Interpreted(_, _, dtype) => dtype,
        }
    }

    pub(self) fn clone_position(&self) -> FilePosition {
        match self {
            Self::Modified(expr, _) => expr.clone_position(),
            Self::Interpreted(_, pos, _) => pos.clone(),
        }
    }

    /// If the result is already an expression, returns that. If the result is interpreted, returns
    /// a literal expression containing the interpreted value.
    pub(self) fn as_vp_expression(self) -> Result<o::VPExpression, CompileProblem> {
        match self {
            Self::Modified(expr, ..) => Ok(expr),
            Self::Interpreted(data, pos, dtype) => {
                if let Ok(rdata) = util::resolve_known_data(&data) {
                    Ok(o::VPExpression::Literal(rdata, pos))
                } else {
                    Err(problems::value_not_run_time_compatible(pos, &dtype))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(self) struct SimplifiedVCExpression {
    pub(self) expr: o::VCExpression,
    pub(self) dtype: DataType,
}

#[derive(Clone, Debug)]
pub(self) enum SimplifiedStatement {
    /// A simpler or resolved version of the statement was found.
    Modified(o::Statement),
    /// The statement had an effect that was entirely predictable at compile time.
    Interpreted,
}
