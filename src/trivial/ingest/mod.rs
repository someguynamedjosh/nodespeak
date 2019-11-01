use crate::problem::CompileProblem;
use crate::trivial::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;
use std::collections::HashMap;

mod problems;

pub fn ingest(program: &i::Program) -> Result<o::Program, CompileProblem> {
    let mut trivializer = Trivializer::new(program);
    trivializer.entry_point()?;
    Result::Ok(trivializer.target)
}

struct Trivializer<'a> {
    source: &'a i::Program,
    target: o::Program,
    variable_map: HashMap<i::VariableId, o::VariableId>,
}

impl<'a> Trivializer<'a> {
    fn new(source: &i::Program) -> Trivializer {
        Trivializer {
            source,
            target: o::Program::new(),
            variable_map: HashMap::new(),
        }
    }

    fn entry_point(&mut self) -> Result<(), CompileProblem> {
        let source_entry_point = self.source.get_entry_point();
        for expr in self.source[source_entry_point].borrow_body().clone() {
            self.trivialize_expression(&expr)?;
        }
        Result::Ok(())
    }

    fn trivialize_known_data(data: &i::KnownData) -> Result<o::LiteralData, CompileProblem> {
        Result::Ok(match data {
            i::KnownData::Int(value) => o::LiteralData::Int(*value),
            i::KnownData::Float(value) => o::LiteralData::Float(*value),
            i::KnownData::Bool(value) => o::LiteralData::Bool(*value),
            _ => panic!("TODO: nice error, encountered ct-only data type after simplification."),
        })
    }

    fn trivialize_data_type(
        data_type: &i::DataType,
    ) -> Result<(o::VariableType, Vec<u64>), CompileProblem> {
        Result::Ok((
            match data_type.borrow_base() {
                i::BaseType::Float => o::VariableType::F32,
                i::BaseType::Int => o::VariableType::I32,
                i::BaseType::Bool => unimplemented!(),
                _ => unreachable!(),
            },
            {
                let mut results = Vec::new();
                for dimension in data_type.borrow_dimensions().iter() {
                    match dimension {
                        i::Expression::Literal(value, position) => match value {
                            i::KnownData::Int(value) => results.push(*value as u64),
                            _ => {
                                return Result::Err(problems::array_size_not_int(
                                    position.clone(),
                                    &value,
                                ))
                            }
                        },
                        _ => {
                            return Result::Err(problems::vague_array_size(
                                dimension.clone_position(),
                            ))
                        }
                    }
                }
                results
            },
        ))
    }

    fn trivialize_variable(
        &mut self,
        variable: i::VariableId,
    ) -> Result<o::VariableId, CompileProblem> {
        Result::Ok(match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let (var_type, dims) = Self::trivialize_data_type(data_type)?;
                let trivial_variable = o::Variable::new_array(var_type, dims);
                let id = self.target.adopt_variable(trivial_variable);
                self.variable_map.insert(variable, id);
                id
            }
        })
    }

    fn trivialize_binary_expression(
        &mut self,
        expression: &i::Expression,
        left: &i::Expression,
        operator: i::BinaryOperator,
        right: &i::Expression,
    ) -> Result<o::Value, CompileProblem> {
        let a = self.trivialize_and_require_value(left, expression)?;
        let b = self.trivialize_and_require_value(right, expression)?;
        let typ = a.get_type(&self.target);
        let x = o::Value::Variable(self.target.adopt_variable(o::Variable::new(typ)));
        let x2 = x.clone();
        let toperator = match operator {
            i::BinaryOperator::Add => match typ {
                o::VariableType::F32 => o::BinaryOperator::AddF,
                o::VariableType::I32 => o::BinaryOperator::AddI,
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Subtract => match typ {
                o::VariableType::F32 => o::BinaryOperator::SubF,
                o::VariableType::I32 => o::BinaryOperator::SubI,
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Multiply => match typ {
                o::VariableType::F32 => o::BinaryOperator::MulF,
                o::VariableType::I32 => o::BinaryOperator::MulI,
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Divide => match typ {
                o::VariableType::F32 => o::BinaryOperator::DivF,
                o::VariableType::I32 => unimplemented!(),
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::IntDiv => match typ {
                o::VariableType::F32 => unimplemented!(),
                o::VariableType::I32 => o::BinaryOperator::DivI,
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Modulo => match typ {
                o::VariableType::F32 => o::BinaryOperator::ModF,
                o::VariableType::I32 => o::BinaryOperator::ModI,
                o::VariableType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Power => unimplemented!(),

            i::BinaryOperator::Equal => unimplemented!(),
            i::BinaryOperator::NotEqual => unimplemented!(),
            i::BinaryOperator::GreaterThan => unimplemented!(),
            i::BinaryOperator::GreaterThanOrEqual => unimplemented!(),
            i::BinaryOperator::LessThan => unimplemented!(),
            i::BinaryOperator::LessThanOrEqual => unimplemented!(),

            i::BinaryOperator::And => unimplemented!(),
            i::BinaryOperator::Or => unimplemented!(),
            i::BinaryOperator::Xor => unimplemented!(),
            i::BinaryOperator::BAnd => unimplemented!(),
            i::BinaryOperator::BOr => unimplemented!(),
            i::BinaryOperator::BXor => unimplemented!(),
        };

        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                op: toperator,
                a,
                b,
                x,
            });

        Result::Ok(x2)
    }

    fn trivialize_and_require_value(
        &mut self,
        expression: &i::Expression,
        part_of: &i::Expression,
    ) -> Result<o::Value, CompileProblem> {
        match self.trivialize_expression(expression)? {
            Some(value) => Result::Ok(value),
            None => Result::Err(problems::void_value(
                expression.clone_position(),
                part_of.clone_position(),
            )),
        }
    }

    fn trivialize_assignment(
        &mut self,
        expression: &i::Expression,
        target: &Box<i::Expression>,
        value: &Box<i::Expression>,
    ) -> Result<(), CompileProblem> {
        let tvalue = self.trivialize_and_require_value(value.borrow(), expression)?;
        let ttarget = self.trivialize_and_require_value(target.borrow(), expression)?;
        self.target.add_instruction(o::Instruction::Move {
            from: tvalue,
            to: ttarget,
        });
        Result::Ok(())
    }

    fn trivialize_for_jump(
        &mut self,
        expression: &i::Expression,
    ) -> Result<o::Condition, CompileProblem> {
        Result::Ok(match expression {
            i::Expression::BinaryOperation(lhs, op, rhs, ..) => {
                let a = self.trivialize_and_require_value(lhs, expression)?;
                let b = self.trivialize_and_require_value(rhs, expression)?;
                // TODO: Error if type of A and B is not bool.
                let condition = match op {
                    i::BinaryOperator::Equal => o::Condition::Equal,
                    i::BinaryOperator::NotEqual => o::Condition::NotEqual,
                    i::BinaryOperator::GreaterThan => o::Condition::GreaterThan,
                    i::BinaryOperator::GreaterThanOrEqual => o::Condition::GreaterThanOrEqual,
                    i::BinaryOperator::LessThan => o::Condition::LessThan,
                    i::BinaryOperator::LessThanOrEqual => o::Condition::LessThanOrEqual,
                    _ => panic!("TODO: Nice error, operation does not return a bool."),
                };
                self.target
                    .add_instruction(o::Instruction::Compare { a, b });
                condition
            }
            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::Variable(..) => unimplemented!(),
            i::Expression::Access { .. } => unimplemented!(),
            _ => panic!("TODO: Nice error, operation does not return a bool."),
        })
    }

    fn trivialize_expression(
        &mut self,
        expression: &i::Expression,
    ) -> Result<Option<o::Value>, CompileProblem> {
        Result::Ok(match expression {
            i::Expression::Literal(data, ..) => {
                Option::Some(o::Value::Literal(Self::trivialize_known_data(data)?))
            }
            i::Expression::Variable(id, ..) => {
                Option::Some(o::Value::Variable(self.trivialize_variable(*id)?))
            }
            i::Expression::Proxy { .. } => unimplemented!(),
            i::Expression::Access { .. } => unimplemented!(),
            i::Expression::InlineReturn(..) => unreachable!("Should be handled elsewhere."),

            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::BinaryOperation(left, operator, right, ..) => {
                Option::Some(self.trivialize_binary_expression(expression, left, *operator, right)?)
            }

            i::Expression::PickInput(..) => unreachable!("trivialize called on unsimplified code."),
            i::Expression::PickOutput(..) => {
                unreachable!("trivialize called on unsimplified code.")
            }
            i::Expression::Collect(..) => unimplemented!(),
            i::Expression::CreationPoint(..) => {
                unreachable!("trivialize called on unsimplified code.")
            }

            i::Expression::Assert(value, ..) => {
                let condition = self.trivialize_for_jump(value)?;
                self.target
                    .add_instruction(o::Instruction::Assert(condition));
                None
            }
            i::Expression::Assign { target, value, .. } => {
                self.trivialize_assignment(expression, target, value)?;
                None
            }
            i::Expression::Return(..) => unimplemented!(),

            i::Expression::FuncCall {
                function,
                setup,
                teardown,
                ..
            } => {
                let mut output_value = None;

                for expr in setup {
                    self.trivialize_expression(expr)?;
                }
                let scope = match function.borrow() {
                    i::Expression::Literal(data, ..) => match data {
                        i::KnownData::Function(data) => data.get_body(),
                        _ => panic!("TODO: nice error, target of func call must be function."),
                    },
                    _ => panic!("TODO: nice error, vague function."),
                };
                for expr in self.source[scope].borrow_body().clone() {
                    self.trivialize_expression(&expr)?;
                }

                for expr in teardown {
                    match expr {
                        i::Expression::InlineReturn(value, ..) => {
                            output_value = Option::Some(
                                self.trivialize_and_require_value(value.borrow(), expr)?,
                            )
                        }
                        _ => {
                            self.trivialize_expression(expr)?;
                        }
                    }
                }

                output_value
            }
        })
    }
}
