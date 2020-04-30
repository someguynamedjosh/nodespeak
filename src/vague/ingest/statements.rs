use super::problems;
use super::VagueIngester;
use crate::ast::structure as i;
use crate::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

impl VagueIngester {
    pub(super) fn convert_function_signature<'a>(
        &mut self,
        node: i::Node<'a>,
    ) -> Result<(Vec<o::VariableId>, Vec<i::Node<'a>>), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::function_signature);
        let mut children = node.into_inner();
        let inputs_node = children.next().expect("illegal grammar");
        let outputs_node = children.next().expect("illegal grammar");
        let input_ids = inputs_node
            .into_inner()
            .map(|child| {
                debug_assert!(child.as_rule() == i::Rule::identifier);
                let name = child.as_str();
                let var = o::Variable::variable(FilePosition::from_pair(&child), None);
                self.target
                    .adopt_and_define_symbol(self.current_scope, name, var)
            })
            .collect();
        Ok((input_ids, outputs_node.into_inner().collect()))
    }

    pub(super) fn convert_function_definition(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::function_definition);
        let position = FilePosition::from_pair(&node);
        let mut children = node.into_inner();
        let func_name_node = children.next().expect("illegal grammar");
        let signature_node = children.next().expect("illegal grammar");
        let header_pos = {
            let mut pos = FilePosition::from_pair(&func_name_node);
            pos.include(&signature_node);
            pos
        };
        let body_node = children.next().expect("illegal grammar");

        let body_scope = self.target.create_child_scope(self.current_scope);
        let old_current_scope = self.current_scope;
        self.current_scope = body_scope;

        let (input_ids, output_nodes) = self.convert_function_signature(signature_node)?;
        for id in input_ids {
            self.target[self.current_scope].add_input(id);
        }
        self.convert_code_block(body_node)?;
        let func_name = func_name_node.as_str();
        for node in output_nodes {
            let name = node.as_str();
            if let Some(id) = self.lookup_identifier_without_error(name) {
                self.target[self.current_scope].add_output(id);
            } else {
                let pos = FilePosition::from_pair(&node);
                return Err(problems::missing_output_definition(pos, func_name, &name));
            };
        }

        self.current_scope = old_current_scope;
        let var = o::Variable::function_def(o::FunctionData::new(body_scope, header_pos));
        let var_id = self
            .target
            .adopt_and_define_symbol(self.current_scope, func_name, var);
        self.add_statement(o::Statement::CreationPoint {
            var: var_id,
            var_type: Box::new(
                o::VPExpression::Literal(o::KnownData::DataType(o::DataType::Function_),
                FilePosition::placeholder(),
            )),
            position
        });
        Ok(())
    }

    pub(super) fn convert_code_block(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::code_block);
        for child in node.into_inner() {
            self.convert_statement(child)?;
        }
        Ok(())
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
        let position = FilePosition::from_pair(&node);
        let mut children = node.into_inner();
        let counter_node = children.next().expect("illegal grammar");
        let counter_pos = FilePosition::from_pair(&counter_node);
        let counter_name = counter_node.as_str();
        let start = self.convert_vpe(children.next().expect("illegal grammar"))?;
        let end = self.convert_vpe(children.next().expect("illegal grammar"))?;
        let body_scope = self.target.create_child_scope(self.current_scope);
        let counter = o::Variable::variable(counter_pos.clone(), None);
        let counter_id = self
            .target
            .adopt_and_define_symbol(body_scope, counter_name, counter);
        self.add_statement(o::Statement::CreationPoint {
            position: counter_pos,
            var: counter_id,
            var_type: Box::new(o::VPExpression::Literal(
                o::KnownData::DataType(o::DataType::Int),
                FilePosition::placeholder(),
            )),
        });
        let old_current_scope = self.current_scope;
        self.current_scope = body_scope;
        self.convert_code_block(children.next().expect("illegal grammar"))?;
        self.current_scope = old_current_scope;
        self.add_statement(o::Statement::ForLoop {
            counter: counter_id,
            start: Box::new(start),
            end: Box::new(end),
            body: body_scope,
            position,
        });
        Ok(())
    }

    pub(super) fn convert_input_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::input_variable_statement);
        if self.current_scope != self.target.get_entry_point() {
            return Err(problems::io_inside_function(FilePosition::from_pair(&node)));
        }
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
        if self.current_scope != self.target.get_entry_point() {
            return Err(problems::io_inside_function(FilePosition::from_pair(&node)));
        }
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
            position,
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