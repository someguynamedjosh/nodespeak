use super::{problems, DataType};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::collections::{HashMap, HashSet};

pub fn ingest(program: &mut i::Program) -> Result<o::Program, CompileProblem> {
    let entry_point = program.get_entry_point();
    let inputs = program[entry_point].borrow_inputs().clone();
    let outputs = program[entry_point].borrow_outputs().clone();
    let old_outputs = outputs.clone();
    let mut resolver = ScopeResolver::new(program);
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

    for input in inputs {
        if let Option::Some(var_id) = input {
            resolver.target.add_input(var_id);
        } else {
            panic!("TODO: Nice error, input not available at run time.")
        }
    }
    for output in outputs {
        if let Option::Some(var_id) = output {
            resolver.target.add_output(var_id);
        } else {
            panic!("TODO: Nice error, input not available at run time.")
        }
    }
    for output in old_outputs {
        if let Some(data) = resolver.borrow_temporary_value(output) {
            let resolved_id = resolver
                .get_var_info(output)
                .expect("Var used before defined, should be caught in vague phase")
                .0
                .expect("TODO: Nice error, output could not be used at runtime");
            let resolved_data = ScopeResolver::resolve_known_data(data)
                .expect("TODO: Nice error, data cannot be used at run time.");
            let p = FilePosition::placeholder();
            resolver.target[new_entry_point].add_statement(o::Statement::Assign {
                target: Box::new(o::VCExpression::variable(resolved_id, p.clone())),
                value: Box::new(o::VPExpression::Literal(resolved_data, p.clone())),
                position: p.clone(),
            });
        }
    }
    resolver.target.set_entry_point(new_entry_point);
    Result::Ok(resolver.target)
}

#[derive(Clone, Debug)]
struct ResolverTable {
    variables: HashMap<i::VariableId, (Option<o::VariableId>, DataType)>,
    unresolved_auto_vars: HashSet<i::VariableId>,
}

impl ResolverTable {
    fn new() -> ResolverTable {
        ResolverTable {
            variables: HashMap::new(),
            unresolved_auto_vars: HashSet::new(),
        }
    }
}

pub(super) struct ScopeResolver<'a> {
    pub(super) source: &'a mut i::Program,
    pub(super) target: o::Program,
    pub(super) current_scope: o::ScopeId,
    table: ResolverTable,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be resolved to
    // different values depending on the types used for the inputs and outputs
    // of the macro.
    stack: Vec<ResolverTable>,
    temp_values: HashMap<i::VariableId, i::KnownData>,
}

impl<'a> ScopeResolver<'a> {
    fn new(source: &'a mut i::Program) -> ScopeResolver<'a> {
        let target = o::Program::new();
        let entry_point = target.get_entry_point();
        ScopeResolver {
            source,
            target,
            current_scope: entry_point,
            table: ResolverTable::new(),
            stack: Vec::new(),
            temp_values: HashMap::new(),
        }
    }

    // Pushes the current state of the conversion table onto the stack. The state
    // can be restored with pop_table().
    pub(super) fn push_table(&mut self) {
        self.stack.push(self.table.clone());
    }

    pub(super) fn pop_table(&mut self) {
        self.table = self
            .stack
            .pop()
            .expect("Encountered extra unexpected stack pop");
    }

    pub(super) fn set_var_info(
        &mut self,
        var: i::VariableId,
        resolved_var: Option<o::VariableId>,
        dtype: DataType,
    ) {
        assert!(
            !self.table.variables.contains_key(&var),
            "Cannot have multiple sets of info for a single variable."
        );
        self.table.variables.insert(var, (resolved_var, dtype));
    }

    pub(super) fn get_var_info(
        &self,
        source: i::VariableId,
    ) -> Option<&(Option<o::VariableId>, DataType)> {
        self.table.variables.get(&source)
    }

    pub(super) fn add_unresolved_auto_var(&mut self, var: i::VariableId) {
        self.table.unresolved_auto_vars.insert(var);
    }

    pub(super) fn is_unresolved_auto_var(&mut self, var: i::VariableId) -> bool {
        self.table.unresolved_auto_vars.contains(&var)
    }

    pub(super) fn mark_auto_var_resolved(&mut self, var: i::VariableId) {
        self.table.unresolved_auto_vars.remove(&var);
    }

    pub(super) fn set_temporary_value(&mut self, var: i::VariableId, value: i::KnownData) {
        self.temp_values.insert(var, value);
    }

    pub(super) fn clear_temporary_value(&mut self, var: i::VariableId) {
        self.temp_values.remove(&var);
    }

    pub(super) fn borrow_temporary_value(&self, var: i::VariableId) -> Option<&i::KnownData> {
        self.temp_values.get(&var)
    }

    pub(super) fn int_literal(value: i64, position: FilePosition) -> o::VPExpression {
        o::VPExpression::Literal(o::KnownData::Int(value), position)
    }

    pub(super) fn resolve_scope(
        &mut self,
        source: i::ScopeId,
    ) -> Result<o::ScopeId, CompileProblem> {
        let old_current_scope = self.current_scope;
        self.current_scope = self.target.create_scope();
        let old_body = self.source[source].borrow_body().clone();
        for statement in old_body {
            if let ResolvedStatement::Modified(new) = self.resolve_statement(&statement)? {
                self.target[self.current_scope].add_statement(new);
            }
        }
        let result = Result::Ok(self.current_scope);
        self.current_scope = old_current_scope;
        result
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedVPExpression {
    /// A simpler or resolved version of the expression was found.
    Modified(o::VPExpression, DataType),
    /// The entire value of the expression has a determinate value.
    Interpreted(i::KnownData, FilePosition, DataType),
}

impl ResolvedVPExpression {
    pub(super) fn borrow_data_type(&self) -> &DataType {
        match self {
            Self::Modified(_, dtype) => dtype,
            Self::Interpreted(_, _, dtype) => dtype,
        }
    }

    pub(super) fn clone_position(&self) -> FilePosition {
        match self {
            Self::Modified(expr, _) => expr.clone_position(),
            Self::Interpreted(_, pos, _) => pos.clone(),
        }
    }

    /// If the result is already an expression, returns that. If the result is interpreted, returns
    /// a literal expression containing the interpreted value.
    pub(super) fn as_vp_expression(self) -> Result<o::VPExpression, CompileProblem> {
        match self {
            Self::Modified(expr, ..) => Ok(expr),
            Self::Interpreted(data, pos, dtype) => {
                if let Ok(rdata) = ScopeResolver::resolve_known_data(&data) {
                    Ok(o::VPExpression::Literal(rdata, pos))
                } else {
                    Err(problems::value_not_run_time_compatible(pos, &dtype))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedVCExpression {
    /// We are not sure what variable / element the VCE is targeting.
    Modified(o::VCExpression, DataType),
    /// We know exactly what variable / element the VCE is targeting.
    Specific(i::VariableId, FilePosition, DataType),
}

impl ResolvedVCExpression {
    pub(super) fn borrow_data_type(&self) -> &DataType {
        match self {
            Self::Modified(_, dtype) => dtype,
            Self::Specific(_, _, dtype) => dtype,
        }
    }

    pub(super) fn clone_position(&self) -> FilePosition {
        match self {
            Self::Modified(expr, _) => expr.position.clone(),
            Self::Specific(_, pos, _) => pos.clone(),
        }
    }

    /// If the result is already an expression, returns that. If the result is specific, returns
    /// a VCE referencing the specific variable.
    pub(super) fn as_vc_expression(
        self,
        resolver: &ScopeResolver,
    ) -> Result<o::VCExpression, CompileProblem> {
        match self {
            Self::Modified(expr, ..) => Ok(expr),
            Self::Specific(in_id, pos, ..) => {
                let (var_id, _var_type) = resolver.get_var_info(in_id).expect(
                    "Variable used before defined, should have been caught in vague phase.",
                );
                let var_id = var_id.expect("TODO: nice error, variable not available at runtime.");
                Ok(o::VCExpression::variable(var_id, pos))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedStatement {
    /// A simpler or resolved version of the statement was found.
    Modified(o::Statement),
    /// The statement had an effect that was entirely predictable at compile time.
    Interpreted,
}
