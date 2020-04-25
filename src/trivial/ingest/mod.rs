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

    fn trivialize_known_data(data: &i::KnownData) -> Result<o::KnownData, CompileProblem> {
        Result::Ok(match data {
            i::KnownData::Int(value) => o::KnownData::Int(*value),
            i::KnownData::Float(value) => o::KnownData::Float(*value),
            i::KnownData::Bool(value) => o::KnownData::Bool(*value),
            _ => panic!("TODO: nice error, encountered ct-only data type after resolving."),
        })
    }

    fn trivialize_data_type(data_type: &i::DataType) -> Result<o::DataType, CompileProblem> {
        let base_type = match data_type.borrow_base() {
            i::BaseType::Float => o::BaseType::F32,
            i::BaseType::Int => o::BaseType::I32,
            i::BaseType::Bool => o::BaseType::B1,
        };
        let dimensions = data_type
            .borrow_dimensions()
            .iter()
            .map(|dim| dim.0)
            .collect();
        Result::Ok(o::DataType::array(base_type, dimensions))
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
                let trivial_variable = o::Variable::new(typ);
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
                base.dimensions = dimensions.clone();
                base
            }
            i::Expression::Literal(data, ..) => {
                let int_dimensions = dimensions.iter().map(|(length, ..)| *length).collect();
                let new_data = match data {
                    i::KnownData::Array(nvec) => {
                        o::KnownData::Array(util::NVec::build_ok(int_dimensions, |index| {
                            let real_index = s::apply_proxy_to_index(&dimensions[..], &index[..]);
                            let item = nvec.borrow_item(&real_index);
                            Self::trivialize_known_data(item)
                        })?)
                    }
                    _ => {
                        if int_dimensions.len() == 0 {
                            Self::trivialize_known_data(data)?
                        } else {
                            o::KnownData::Array(util::NVec::new(
                                int_dimensions.iter().map(|e| *e as usize).collect(),
                                Self::trivialize_known_data(data)?,
                            ))
                        }
                    }
                };
                o::Value::literal(new_data)
            }
            _ => {
                // TODO: Figure out what to provide for the part_of argument.
                let mut base = self.trivialize_and_require_value(base, base)?;
                base.dimensions = dimensions.clone();
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
        let dimensions: Vec<_> = a.dimensions.iter().map(|(size, _)| *size).collect();
        let typ = o::DataType::array(base_type, dimensions.clone());
        let base = typ.get_base();
        let out_base = match operator {
            i::BinaryOperator::Add
            | i::BinaryOperator::Subtract
            | i::BinaryOperator::Multiply
            | i::BinaryOperator::Divide
            | i::BinaryOperator::Modulo
            | i::BinaryOperator::Power => base,
            i::BinaryOperator::Equal
            | i::BinaryOperator::NotEqual
            | i::BinaryOperator::GreaterThan
            | i::BinaryOperator::LessThan
            | i::BinaryOperator::GreaterThanOrEqual
            | i::BinaryOperator::LessThanOrEqual => o::BaseType::B1,
            _ => unimplemented!(),
        };
        let out_typ = o::DataType::array(out_base, dimensions);
        let x = o::Value::variable(self.target.adopt_variable(o::Variable::new(out_typ)));
        let x2 = x.clone();
        let toperator = match operator {
            i::BinaryOperator::Add => match base {
                o::BaseType::F32 => o::BinaryOperator::AddF,
                o::BaseType::I32 => o::BinaryOperator::AddI,
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::Subtract => match base {
                o::BaseType::F32 => o::BinaryOperator::SubF,
                o::BaseType::I32 => o::BinaryOperator::SubI,
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::Multiply => match base {
                o::BaseType::F32 => o::BinaryOperator::MulF,
                o::BaseType::I32 => o::BinaryOperator::MulI,
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::Divide => match base {
                o::BaseType::F32 => o::BinaryOperator::DivF,
                o::BaseType::I32 => o::BinaryOperator::DivI,
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::Modulo => match base {
                o::BaseType::F32 => o::BinaryOperator::ModF,
                o::BaseType::I32 => o::BinaryOperator::ModI,
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::Power => unimplemented!(),

            i::BinaryOperator::Equal => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::Equal),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::Equal),
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::NotEqual => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::NotEqual),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::NotEqual),
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::GreaterThan => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThan),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThan),
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::GreaterThanOrEqual => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThanOrEqual),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThanOrEqual),
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::LessThan => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::LessThan),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::LessThan),
                o::BaseType::B1 => unimplemented!(),
            },
            i::BinaryOperator::LessThanOrEqual => match base {
                o::BaseType::F32 => o::BinaryOperator::CompF(o::Condition::LessThanOrEqual),
                o::BaseType::I32 => o::BinaryOperator::CompI(o::Condition::LessThanOrEqual),
                o::BaseType::B1 => unimplemented!(),
            },

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
        target: &i::Expression,
        value: &i::Expression,
    ) -> Result<(), CompileProblem> {
        let tvalue = self.trivialize_and_require_value(value.borrow(), expression)?;
        if let i::Expression::Access { base, indexes, .. } = target {
            let base = if let i::Expression::Variable(id, ..) = base.borrow() {
                self.trivialize_variable(*id)?
            } else {
                panic!("TODO: Check if it is guaranteed that a variable is always the base.");
            };
            let (new_indexes, _result_type) =
                self.trivialize_indexes(indexes, self.target[base].borrow_type().clone())?;
            self.target.add_instruction(o::Instruction::Store {
                from: tvalue,
                to: o::Value::variable(base),
                to_indexes: new_indexes,
            });
        } else {
            let ttarget = self.trivialize_and_require_value(target.borrow(), expression)?;
            self.target.add_instruction(o::Instruction::Move {
                from: tvalue,
                to: ttarget,
            });
        }
        Result::Ok(())
    }

    fn trivialize_branch(
        &mut self,
        clauses: &Vec<(i::Expression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
    ) -> Result<(), CompileProblem> {
        debug_assert!(clauses.len() > 0);
        let end_label = self.target.create_label();
        for (condition_expr, body) in clauses.iter() {
            // TODO: Figure out what to do for the part_of argument.
            let condition = self.trivialize_and_require_value(condition_expr, condition_expr)?;
            let body_label = self.target.create_label();
            let next_condition_label = self.target.create_label();
            self.target.add_instruction(o::Instruction::Branch {
                condition,
                true_target: Some(body_label),
                false_target: Some(next_condition_label),
            });
            self.target
                .add_instruction(o::Instruction::Label(body_label));
            for expr in self.source[*body].borrow_body().clone() {
                self.trivialize_expression(&expr)?;
            }
            self.target
                .add_instruction(o::Instruction::Jump { label: end_label });
            self.target
                .add_instruction(o::Instruction::Label(next_condition_label));
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
        let (start_label, end_label) = (self.target.create_label(), self.target.create_label());
        // TODO: Better value for part_of.
        let tcount = o::Value::variable(self.trivialize_variable(counter)?);
        let tstart = self.trivialize_and_require_value(start, start)?;
        let tend = self.trivialize_and_require_value(end, end)?;
        self.target.add_instruction(o::Instruction::Move {
            from: tstart,
            to: tcount.clone(),
        });
        let condition_var = o::Value::variable(
            self.target
                .adopt_variable(o::Variable::new(o::DataType::b1_scalar())),
        );
        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                a: tcount.clone(),
                b: tend.clone(),
                x: condition_var.clone(),
                op: o::BinaryOperator::CompI(o::Condition::LessThan),
            });
        self.target.add_instruction(o::Instruction::Branch {
            condition: condition_var.clone(),
            true_target: Some(end_label),
            false_target: Some(start_label),
        });

        self.target
            .add_instruction(o::Instruction::Label(start_label));
        for expr in self.source[body].borrow_body().clone() {
            self.trivialize_expression(&expr)?;
        }

        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                a: tcount.clone(),
                b: o::Value::literal(o::KnownData::Int(1)),
                x: tcount.clone(),
                op: o::BinaryOperator::AddI,
            });
        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                a: tcount.clone(),
                b: tend,
                x: condition_var.clone(),
                op: o::BinaryOperator::CompI(o::Condition::LessThan),
            });
        self.target.add_instruction(o::Instruction::Branch {
            condition: condition_var.clone(),
            true_target: Some(end_label),
            false_target: Some(start_label),
        });
        self.target
            .add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_indexes(
        &mut self,
        indexes: &Vec<i::Expression>,
        data_type: o::DataType,
    ) -> Result<(Vec<o::Value>, o::DataType), CompileProblem> {
        let mut tindexes = Vec::with_capacity(indexes.len());
        let mut element_type = data_type;
        for index in indexes {
            // TODO: Better value for part_of argument.
            let index_value = self.trivialize_and_require_value(index, index)?;
            let indext = index_value.get_type(&self.target);
            if !indext.is_int() || !indext.is_scalar() {
                panic!("TODO: Nice error, index is not int scalar.");
            }
            tindexes.push(index_value);
            element_type = element_type.clone_and_unwrap(1);
        }
        Ok((tindexes, element_type))
    }

    fn trivialize_access(
        &mut self,
        base: &i::Expression,
        indexes: &Vec<i::Expression>,
    ) -> Result<o::Value, CompileProblem> {
        let base = match base.borrow() {
            i::Expression::Variable(id, ..) => self.trivialize_variable(*id)?,
            i::Expression::Proxy { .. } => unimplemented!(),
            _ => panic!("TODO: nice error, invalid base for access: {:?}", base),
        };
        let (new_indexes, result_type) =
            self.trivialize_indexes(indexes, self.target[base].borrow_type().clone())?;

        let output_holder = self
            .target
            .adopt_variable(o::Variable::new(result_type.clone()));
        let mut output_value = o::Value::variable(output_holder);
        output_value.dimensions = result_type
            .borrow_dimensions()
            .iter()
            .map(|dim| (*dim, s::ProxyMode::Keep))
            .collect();
        self.target.add_instruction(o::Instruction::Load {
            from: o::Value::variable(base),
            to: output_value.clone(),
            from_indexes: new_indexes,
        });
        Ok(output_value)
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
                Some(self.trivialize_access(base, indexes)?)
            }
            i::Expression::InlineReturn(..) => unreachable!("Should be handled elsewhere."),

            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::BinaryOperation(left, operator, right, ..) => {
                Option::Some(self.trivialize_binary_expression(expression, left, *operator, right)?)
            }

            i::Expression::Collect(..) => unimplemented!(),

            i::Expression::Assert(..) => unimplemented!(),
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
