use crate::problem::CompileProblem;
use crate::resolved::structure as i;
use crate::shared as s;
use crate::trivial::structure as o;
use crate::util;
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
        let source_inputs = self.source[source_entry_point].borrow_inputs().clone();
        for input in source_inputs {
            let trivialized = self.trivialize_variable(input)?;
            self.target.add_input(trivialized);
        }
        let source_outputs = self.source[source_entry_point].borrow_outputs().clone();
        for output in source_outputs {
            let trivialized = self.trivialize_variable(output)?;
            self.target.add_output(trivialized);
        }
        for expr in self.source[source_entry_point].borrow_body().clone() {
            self.trivialize_expression(&expr)?;
        }
        Result::Ok(())
    }

    fn trivialize_known_data(data: &i::KnownData) -> Result<s::NativeData, CompileProblem> {
        Result::Ok(match data {
            i::KnownData::Int(value) => s::NativeData::Int(*value),
            i::KnownData::Float(value) => s::NativeData::Float(*value),
            i::KnownData::Bool(value) => s::NativeData::Bool(*value),
            _ => panic!("TODO: nice error, encountered ct-only data type after resolving."),
        })
    }

    fn trivialize_data_type(data_type: &i::DataType) -> Result<s::NativeType, CompileProblem> {
        let base_type = match data_type.borrow_base() {
            i::BaseType::Float => s::NativeBaseType::F32,
            i::BaseType::Int => s::NativeBaseType::I32,
            i::BaseType::Bool => s::NativeBaseType::B8,
        };
        let dimensions = data_type.borrow_dimensions().clone();
        Result::Ok(s::NativeType::array(base_type, dimensions))
    }

    fn trivialize_variable(
        &mut self,
        variable: i::VariableId,
    ) -> Result<o::VariableId, CompileProblem> {
        Result::Ok(match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let typ = Self::trivialize_data_type(data_type)?;
                let trivial_variable = s::NativeVar::new(typ);
                let id = self.target.adopt_variable(trivial_variable);
                self.variable_map.insert(variable, id);
                id
            }
        })
    }

    fn trivialize_proxy(
        &mut self,
        base: &i::Expression,
        dimensions: &Vec<(usize, s::ProxyMode)>,
    ) -> Result<o::Value, CompileProblem> {
        // TODO: Check that the proxy is actually valid.
        Ok(match base {
            i::Expression::Variable(id, ..) => {
                let mut base = o::Value::variable(self.trivialize_variable(*id)?);
                for (size, mode) in dimensions {
                    base.proxy.push((mode.clone(), *size));
                }
                base
            }
            i::Expression::Literal(data, ..) => {
                let int_dimensions = dimensions.iter().map(|(length, ..)| *length).collect();
                let new_data = match data {
                    i::KnownData::Array(nvec) => {
                        s::NativeData::Array(util::NVec::build_ok(int_dimensions, |index| {
                            let real_index = s::apply_proxy_to_index(&dimensions[..], &index[..]);
                            let item = nvec.borrow_item(&real_index);
                            Self::trivialize_known_data(item)
                        })?)
                    }
                    _ => {
                        if int_dimensions.len() == 0 {
                            Self::trivialize_known_data(data)?
                        } else {
                            s::NativeData::Array(util::NVec::new(
                                int_dimensions.iter().map(|e| *e as usize).collect(),
                                Self::trivialize_known_data(data)?,
                            ))
                        }
                    }
                };
                o::Value::literal(new_data)
            }
            _ => {
                // TODO: Figure out what to do with the part_of argument.
                let mut base = self.trivialize_and_require_value(base, base)?;
                for (size, mode) in dimensions {
                    base.proxy.push((mode.clone(), *size));
                }
                base
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
        let base_type = a.get_type(&self.target).get_base();
        let dimensions = a
            .borrow_proxy()
            .iter()
            .map(|(_, size)| (*size, s::ProxyMode::Keep))
            .collect();
        let typ = s::NativeType::array(base_type, dimensions);
        let base = typ.get_base();
        let x = o::Value::variable(self.target.adopt_variable(s::NativeVar::new(typ)));
        let x2 = x.clone();
        let toperator = match operator {
            i::BinaryOperator::Add => match base {
                s::NativeBaseType::F32 => o::BinaryOperator::AddF,
                s::NativeBaseType::I32 => o::BinaryOperator::AddI,
                s::NativeBaseType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Subtract => match base {
                s::NativeBaseType::F32 => o::BinaryOperator::SubF,
                s::NativeBaseType::I32 => o::BinaryOperator::SubI,
                s::NativeBaseType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Multiply => match base {
                s::NativeBaseType::F32 => o::BinaryOperator::MulF,
                s::NativeBaseType::I32 => o::BinaryOperator::MulI,
                s::NativeBaseType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Divide => match base {
                s::NativeBaseType::F32 => o::BinaryOperator::DivF,
                s::NativeBaseType::I32 => o::BinaryOperator::DivI,
                s::NativeBaseType::B8 => unimplemented!(),
            },
            i::BinaryOperator::Modulo => match base {
                s::NativeBaseType::F32 => o::BinaryOperator::ModF,
                s::NativeBaseType::I32 => o::BinaryOperator::ModI,
                s::NativeBaseType::B8 => unimplemented!(),
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

    fn trivialize_branch(
        &mut self,
        clauses: &Vec<(i::Expression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
    ) -> Result<(), CompileProblem> {
        debug_assert!(clauses.len() > 0);
        let end_label = self.target.create_label(false);
        for (condition_expr, body) in clauses.iter() {
            let skip_condition = self.trivialize_for_jump(condition_expr)?.as_negative();
            let next_clause_label = self.target.create_label(true);
            self.target
                .add_instruction(o::Instruction::ConditionalJump {
                    condition: skip_condition,
                    label: next_clause_label,
                });
            for expr in self.source[*body].borrow_body().clone() {
                self.trivialize_expression(&expr)?;
            }
            self.target
                .add_instruction(o::Instruction::Jump { label: end_label });
            self.target
                .add_instruction(o::Instruction::Label(next_clause_label));
        }
        if let Some(body) = /* once told me */ else_clause {
            for expr in self.source[*body].borrow_body().clone() {
                self.trivialize_expression(&expr)?;
            }
        }
        self.target
            .add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_for_loop(
        &mut self,
        counter: i::VariableId,
        start: &i::Expression,
        end: &i::Expression,
        body: i::ScopeId,
    ) -> Result<(), CompileProblem> {
        let (start_label, end_label) = (self.target.create_label(false), self.target.create_label(false));
        // TODO: Better value for part_of.
        let tcount = o::Value::variable(self.trivialize_variable(counter)?);
        let tstart = self.trivialize_and_require_value(start, start)?;
        let tend = self.trivialize_and_require_value(end, end)?;
        self.target.add_instruction(o::Instruction::Move {
            from: tstart,
            to: tcount.clone(),
        });
        self.target
            .add_instruction(o::Instruction::Label(start_label));
        self.target
            .add_instruction(o::Instruction::Compare { a: tcount.clone(), b: tend });
        self.target
            .add_instruction(o::Instruction::ConditionalJump {
                condition: o::Condition::GreaterThanOrEqual,
                label: end_label,
            });
        for expr in self.source[body].borrow_body().clone() {
            self.trivialize_expression(&expr)?;
        }
        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                a: tcount.clone(),
                b: o::Value::literal(s::NativeData::Int(1)),
                x: tcount,
                op: o::BinaryOperator::AddI,
            });
        self.target
            .add_instruction(o::Instruction::Jump { label: start_label });
        self.target.add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_expression(
        &mut self,
        expression: &i::Expression,
    ) -> Result<Option<o::Value>, CompileProblem> {
        Result::Ok(match expression {
            i::Expression::Literal(data, ..) => {
                Option::Some(o::Value::literal(Self::trivialize_known_data(data)?))
            }
            i::Expression::Variable(id, ..) => {
                Option::Some(o::Value::variable(self.trivialize_variable(*id)?))
            }
            i::Expression::Proxy {
                base, dimensions, ..
            } => Some(self.trivialize_proxy(base, dimensions)?),
            i::Expression::Access { base, indexes, .. } => {
                // TODO: Check that indexes are integers.
                let base = match base.borrow() {
                    i::Expression::Variable(id, ..) => self.trivialize_variable(*id)?,
                    i::Expression::Proxy { .. } => unimplemented!(),
                    _ => panic!("TODO: nice error, invalid base for access: {:?}", base),
                };
                let mut new_indexes = Vec::new();
                for index in indexes {
                    let index_value = self.trivialize_and_require_value(index, expression)?;
                    if index_value.borrow_proxy().len() == 0 && index_value.indexes.len() == 0 {
                        // TODO: Check that index is integer.
                        new_indexes.push(index_value.base);
                    } else {
                        unimplemented!("Encountered indexed value inside index expression.");
                    }
                }
                let mut value = o::Value::variable(base);
                value.indexes = new_indexes;
                Option::Some(value)
            }
            i::Expression::InlineReturn(..) => unreachable!("Should be handled elsewhere."),

            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::BinaryOperation(left, operator, right, ..) => {
                Option::Some(self.trivialize_binary_expression(expression, left, *operator, right)?)
            }

            i::Expression::Collect(..) => unimplemented!(),

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
            i::Expression::Branch {
                clauses,
                else_clause,
                ..
            } => {
                self.trivialize_branch(clauses, else_clause)?;
                None
            }
            i::Expression::ForLoop {
                counter,
                start,
                end,
                body,
                ..
            } => {
                self.trivialize_for_loop(*counter, start, end, *body)?;
                None
            }

            i::Expression::FuncCall { function, .. } => {
                for expr in self.source[*function].borrow_body().clone() {
                    self.trivialize_expression(&expr)?;
                }
                None
            }
        })
    }
}
