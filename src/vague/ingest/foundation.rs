use super::problems;
use crate::ast::structure as i;
use crate::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

pub(super) struct VagueIngester {
    pub(super) target: o::Program,
    pub(super) current_scope: o::ScopeId,
}

impl VagueIngester {
    pub(super) fn lookup_identifier(
        &self,
        node: &i::Node,
    ) -> Result<o::VariableId, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::identifier);
        match self.target.lookup_symbol(self.current_scope, node.as_str()) {
            Option::Some(entity) => Result::Ok(entity),
            Option::None => {
                let position = FilePosition::from_pair(node);
                Result::Err(problems::no_entity_with_name(position))
            }
        }
    }

    pub(super) fn add_statement(&mut self, statement: o::Statement) {
        self.target[self.current_scope].add_statement(statement);
    }

    pub(super) fn create_variable(
        &mut self,
        data_type: o::VPExpression,
        name: &str,
        decl_pos: FilePosition,
    ) -> o::VariableId {
        let var = o::Variable::variable(decl_pos.clone(), None);
        let var_id = self
            .target
            .adopt_and_define_symbol(self.current_scope, name, var);
        self.add_statement(o::Statement::CreationPoint {
            var: var_id,
            var_type: Box::new(data_type),
            position: decl_pos,
        });
        var_id
    }

    fn execute(&mut self, source: &mut i::Program) -> Result<(), CompileProblem> {
        let root_node = source.next().expect("illegal grammar");
        debug_assert!(root_node.as_rule() == i::Rule::root);
        for child in root_node.into_inner() {
            if child.as_rule() == i::Rule::EOI {
                break;
            }
            self.convert_statement(child)?;
        }
        Ok(())
    }
}

pub fn ingest(source: &mut i::Program) -> Result<o::Program, CompileProblem> {
    let target = o::Program::new();
    let init_scope = target.get_entry_point();
    let mut ingester = VagueIngester {
        target,
        current_scope: init_scope,
    };
    ingester.execute(source)?;
    Ok(ingester.target)
}
