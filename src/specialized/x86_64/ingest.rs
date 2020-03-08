use crate::specialized::x86_64::structure as o;
use crate::trivial::structure as i;
use crate::util;
use std::collections::HashMap;

pub fn ingest(input: &i::Program) -> o::Program {
    let mut specializer = Specializer {
        source: input,
        target: o::Program::new(),
        var_conversions: HashMap::new(),
    };
    specializer.entry_point();
    specializer.target
}

struct Specializer<'a> {
    source: &'a i::Program,
    target: o::Program,
    var_conversions: HashMap<i::VariableId, o::VariableId>,
}

impl<'a> Specializer<'a> {
    fn entry_point(&mut self) {
        let instructions = self.source.borrow_instructions().clone();
        for input in self.source.borrow_inputs() {
            let specialized = self.specialize_variable(*input);
            self.target.add_input(specialized);
        }
        for output in self.source.borrow_outputs() {
            let specialized = self.specialize_variable(*output);
            self.target.add_output(specialized);
        }
        for instruction in instructions {
            self.specialize_instruction(instruction);
        }
        self.target.complete_liveness_analysis();
    }

    fn specialize_binary_operator(operator: &i::BinaryOperator) -> o::BinaryOperator {
        match operator {
            i::BinaryOperator::AddI => o::BinaryOperator::AddI,
            i::BinaryOperator::SubI => o::BinaryOperator::SubI,
            i::BinaryOperator::MulI => o::BinaryOperator::MulI,
            i::BinaryOperator::DivI => o::BinaryOperator::DivI,
            i::BinaryOperator::ModI => o::BinaryOperator::ModI,
            i::BinaryOperator::AddF => o::BinaryOperator::AddF,
            i::BinaryOperator::SubF => o::BinaryOperator::SubF,
            i::BinaryOperator::MulF => o::BinaryOperator::MulF,
            i::BinaryOperator::DivF => o::BinaryOperator::DivF,
            i::BinaryOperator::ModF => o::BinaryOperator::ModF,
            i::BinaryOperator::BAnd => o::BinaryOperator::BAnd,
            i::BinaryOperator::BOr => o::BinaryOperator::BOr,
            i::BinaryOperator::BXor => o::BinaryOperator::BXor,
        }
    }

    fn specialize_condition(condition: &i::Condition) -> o::Condition {
        match condition {
            i::Condition::Equal => o::Condition::Equal,
            i::Condition::NotEqual => o::Condition::NotEqual,
            i::Condition::GreaterThan => o::Condition::GreaterThan,
            i::Condition::LessThan => o::Condition::LessThan,
            i::Condition::GreaterThanOrEqual => o::Condition::GreaterThanOrEqual,
            i::Condition::LessThanOrEqual => o::Condition::LessThanOrEqual,
        }
    }

    fn specialize_instruction(&mut self, instruction: &i::Instruction) {
        match instruction {
            i::Instruction::Move { from, to } => {
                let from = self.specialize_value(from);
                let to = self.specialize_value(to);
                self.target
                    .add_instruction(o::Instruction::Move { from, to });
            }
            i::Instruction::BinaryOperation { op, a, b, x } => {
                // TODO: Parallelize when appropriate.
                let a = self.specialize_value(a);
                let b = self.specialize_value(b);
                let x = self.specialize_value(x);
                let op = Self::specialize_binary_operator(op);
                let operation_size = x.get_type(&self.target).borrow_dimensions().clone();
                if operation_size.len() > 0 {
                    let operation_dims = operation_size.iter().map(|(size, _)| *size).collect();
                    for indexes in util::nd_index_iter(operation_dims) {
                        println!("{:?}", indexes);
                        self.target
                            .add_instruction(o::Instruction::BinaryOperation {
                                op: op.clone(),
                                a: a.access_element(&indexes),
                                b: b.access_element(&indexes),
                                x: x.access_element(&indexes),
                            });
                    }
                } else {
                    self.target
                        .add_instruction(o::Instruction::BinaryOperation { op, a, b, x });
                }
            }
            i::Instruction::Compare { a, b } => {
                let a = self.specialize_value(a);
                let b = self.specialize_value(b);
                self.target
                    .add_instruction(o::Instruction::Compare { a, b });
            }
            i::Instruction::Assert(condition) => {
                let condition = Self::specialize_condition(condition);
                self.target
                    .add_instruction(o::Instruction::Assert(condition));
            }
            _ => unimplemented!("{:?}", instruction),
        }
    }

    fn specialize_value(&mut self, value: &i::Value) -> o::Value {
        match &value.base {
            i::ValueBase::Literal(data) => {
                if value.indexes.len() > 0 {
                    unimplemented!()
                } else {
                    o::Value::Literal(data.clone())
                }
            }
            i::ValueBase::Variable(var) => {
                if value.indexes.len() > 0 {
                    unimplemented!()
                } else {
                    let id = self.specialize_variable(*var);
                    o::Value::VariableAccess {
                        variable: id,
                        proxy: value.borrow_proxy().iter().map(|(mode, _)| *mode).collect(),
                        indexes: vec![],
                    }
                }
            }
        }
    }

    fn specialize_variable(&mut self, variable: i::VariableId) -> o::VariableId {
        if let Some(specialized) = self.var_conversions.get(&variable) {
            *specialized
        } else {
            // Technically, the input and output variable structs are the same.
            // TODO: Make sure that none of the data inside it needs to be converted.
            let new_var = self.source[variable].clone();
            let specialized = self.target.adopt_variable(new_var);
            self.var_conversions.insert(variable, specialized);
            specialized
        }
    }
}
