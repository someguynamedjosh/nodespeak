use super::{
    problems, util, DataType, ScopeResolver, ResolvedVCExpression, ResolvedVPExpression,
};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;

impl<'a> ScopeResolver<'a> {
    pub(super) fn resolve_vc_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        let (_, var_type) = self
            .get_var_info(var_id)
            .expect("Variable used before declaration, vague step should have caught this.");
        Ok(ResolvedVCExpression::Specific(var_id, position.clone(), var_type.clone()))
    }

    pub(super) fn resolve_vc_expression(
        &mut self,
        input: &i::VCExpression,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        match input {
            i::VCExpression::Variable(id, position) => self.resolve_vc_variable(*id, position),
            i::VCExpression::Index{..} => unimplemented!(),
        }
    }
}
