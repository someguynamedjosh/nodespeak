use super::problems;
use super::VagueIngester;
use crate::ast::structure as i;
use crate::problem::{FilePosition, CompileProblem};
use crate::vague::structure as o;

impl VagueIngester {
    pub(super) fn convert_function_definition(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::function_definition);
        let mut children = node.into_inner();
        let func_name = children.next().expect("illegal grammar").as_str();
        let signature_node = children.next().expect("illegal grammar");
        let body_node = children.next().expect("illegal grammar");
        unimplemented!();
    }

    pub(super) fn convert_code_block(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::code_block);
        unimplemented!()
    }

    pub(super) fn convert_return_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::return_statement);
        unimplemented!()
    }

    pub(super) fn convert_assert_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::assert_statement);
        unimplemented!()
    }

    pub(super) fn convert_if_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::if_statement);
        unimplemented!()
    }

    pub(super) fn convert_for_loop_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::for_loop_statement);
        unimplemented!()
    }

    pub(super) fn convert_input_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::input_variable_statement);
        let mut children = node.into_inner();
        let data_type = self.convert_vpe(children.next().expect("illegal grammar"))?;
        for child in children {
            let pos = FilePosition::from_pair(&child);
            let name = child.as_str();
            let var_id = self.create_variable(data_type.clone(), name, pos);
            self.target[self.current_scope].add_input(var_id);
        }
        Ok(())
    }

    pub(super) fn convert_output_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::output_variable_statement);
        let mut children = node.into_inner();
        let data_type = self.convert_vpe(children.next().expect("illegal grammar"))?;
        for child in children {
            let pos = FilePosition::from_pair(&child);
            let name = child.as_str();
            let var_id = self.create_variable(data_type.clone(), name, pos);
            self.target[self.current_scope].add_output(var_id);
        }
        Ok(())
    }

    pub(super) fn convert_assign_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::assign_statement);
        let position = FilePosition::from_pair(&node);
        let mut children = node.into_inner();
        let vce = self.convert_vce(children.next().expect("illegal grammar"))?;
        let vpe = self.convert_vpe(children.next().expect("illegal grammar"))?;
        self.add_statement(o::Statement::Assign {
            target: Box::new(vce),
            value: Box::new(vpe),
            position
        });
        Ok(())
    }

    pub(super) fn convert_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::statement);
        let child = node.into_inner().next().expect("illegal grammar");
        match child.as_rule() {
            i::Rule::function_definition => self.convert_function_definition(child)?,
            i::Rule::code_block => self.convert_code_block(child)?,
            i::Rule::return_statement => self.convert_return_statement(child)?,
            i::Rule::assert_statement => self.convert_assert_statement(child)?,
            i::Rule::if_statement => self.convert_if_statement(child)?,
            i::Rule::for_loop_statement => self.convert_for_loop_statement(child)?,
            i::Rule::input_variable_statement => self.convert_input_variable_statement(child)?,
            i::Rule::output_variable_statement => self.convert_output_variable_statement(child)?,
            i::Rule::assign_statement => self.convert_assign_statement(child)?,
            i::Rule::func_call => {
                self.convert_func_call(child, false)?;
            }
            i::Rule::var_dec => {
                self.convert_var_dec(child)?;
            }
            _ => unreachable!("illegal grammar"),
        }
        Ok(())
    }
}
