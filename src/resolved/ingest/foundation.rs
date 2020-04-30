use std::collections::HashMap;

use super::{
    util, DataType, ScopeSimplifier, SimplifiedStatement, SimplifiedVCExpression,
    SimplifiedVPExpression, SimplifierTable,
};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn new(source: &'a mut i::Program) -> ScopeSimplifier<'a> {
        let target = o::Program::new();
        let entry_point = target.get_entry_point();
        ScopeSimplifier {
            source,
            target,
            current_scope: entry_point,
            table: SimplifierTable::new(),
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
            if let SimplifiedStatement::Modified(new) = self.resolve_statement(&statement)? {
                self.target[self.current_scope].add_statement(new);
            }
        }
        let result = Result::Ok(self.current_scope);
        self.current_scope = old_current_scope;
        result
    }
}
