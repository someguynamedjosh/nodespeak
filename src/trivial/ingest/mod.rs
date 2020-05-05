use crate::high_level::problem::CompileProblem;
use crate::resolved::structure as i;
use crate::shared as s;
use crate::trivial::structure as o;
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
        let source_inputs = self.source.borrow_inputs();
        for input in source_inputs {
            let trivialized = self.trivialize_variable(*input)?;
            self.target.add_input(trivialized);
        }
        let source_outputs = self.source.borrow_outputs();
        for output in source_outputs {
            let trivialized = self.trivialize_variable(*output)?;
            self.target.add_output(trivialized);
        }
        for statement in self.source[source_entry_point].borrow_body().clone() {
            self.trivialize_statement(&statement)?;
        }
        Result::Ok(())
    }

    fn bct_dimensions(type1: &o::DataType, type2: &o::DataType) -> Vec<usize> {
        let t1dims = type1.collect_dimensions();
        let t2dims = type2.collect_dimensions();
        let mut bctdims = Vec::new();
        for index in 0..(t1dims.len().max(t2dims.len())) {
            let dim = if index >= t1dims.len() {
                t2dims[index]
            } else if index >= t2dims.len() {
                t1dims[index]
            } else if t1dims[index] == 1 {
                t2dims[index]
            } else if t2dims[index] == 1 {
                t1dims[index]
            } else if t1dims[index] == t2dims[index] {
                t1dims[index]
            } else {
                unreachable!("Invalid bct should have been caught earlier.")
            };
            bctdims.push(dim);
        }
        bctdims
    }

    fn trivialize_known_data(data: &i::KnownData) -> Result<o::KnownData, CompileProblem> {
        Result::Ok(match data {
            i::KnownData::Int(value) => o::KnownData::Int(*value),
            i::KnownData::Float(value) => o::KnownData::Float(*value),
            i::KnownData::Bool(value) => o::KnownData::Bool(*value),
            i::KnownData::Array(items) => {
                let mut titems = Vec::with_capacity(items.len());
                for item in items {
                    titems.push(Self::trivialize_known_data(item)?);
                }
                o::KnownData::Array(titems)
            }
            _ => panic!("TODO: nice error, encountered ct-only data type after resolving"),
        })
    }

    fn trivialize_data_type(data_type: &i::DataType) -> o::DataType {
        match data_type {
            i::DataType::Float => o::DataType::F32,
            i::DataType::Int => o::DataType::I32,
            i::DataType::Bool => o::DataType::B1,
            i::DataType::Array(len, base) => {
                o::DataType::Array(*len, Box::new(Self::trivialize_data_type(base)))
            }
        }
    }

    fn trivialize_variable(
        &mut self,
        variable: i::VariableId,
    ) -> Result<o::VariableId, CompileProblem> {
        Result::Ok(match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let typ = Self::trivialize_data_type(data_type);
                let trivial_variable = o::Variable::new(typ);
                let id = self.target.adopt_variable(trivial_variable);
                self.variable_map.insert(variable, id);
                id
            }
        })
    }

    fn apply_proxy_to_known_data(
        proxy: &[(usize, s::ProxyMode)],
        data: &i::KnownData,
    ) -> Result<o::KnownData, CompileProblem> {
        let int_dimensions: Vec<_> = proxy.iter().map(|(length, ..)| *length).collect();
        Ok(if proxy.len() == 0 {
            Self::trivialize_known_data(data)?
        } else if let i::KnownData::Array(vec) = data {
            let proxy_elements = vec
                .iter()
                .map(|element| Self::apply_proxy_to_known_data(&proxy[1..], element))
                .collect::<Result<Vec<_>, _>>()?;
            if proxy[0].1 == s::ProxyMode::Collapse && proxy_elements.len() == 1 {
                o::KnownData::build_array(&[proxy[0].0], proxy_elements[0].clone())
            } else if proxy[0].1 == s::ProxyMode::Discard {
                o::KnownData::build_array(&[proxy[0].0], o::KnownData::Array(proxy_elements))
            } else if proxy[0].1 == s::ProxyMode::Keep {
                o::KnownData::Array(proxy_elements)
            } else {
                panic!("TODO: Nice error, invalid proxy for data.");
            }
        } else {
            o::KnownData::build_array(&int_dimensions[..], Self::trivialize_known_data(data)?)
        })
    }

    fn trivialize_binary_expression(
        &mut self,
        left: &i::VPExpression,
        operator: i::BinaryOperator,
        right: &i::VPExpression,
        out_typ: &i::DataType,
    ) -> Result<o::Value, CompileProblem> {
        let mut a = self.trivialize_vp_expression(left)?;
        let mut b = self.trivialize_vp_expression(right)?;
        let bct_dims = Self::bct_dimensions(&a.get_type(&self.target), &b.get_type(&self.target));
        a.inflate(&bct_dims[..]);
        b.inflate(&bct_dims[..]);
        let out_typ = Self::trivialize_data_type(out_typ);
        let mut base = a.get_type(&self.target);
        while let o::DataType::Array(_, etype) = base {
            base = *etype;
        }
        let x_var = self.target.adopt_variable(o::Variable::new(out_typ));
        let x = o::Value::variable(x_var, &self.target);
        let x2 = x.clone();
        let toperator = match operator {
            i::BinaryOperator::Add => match base {
                o::DataType::F32 => o::BinaryOperator::AddF,
                o::DataType::I32 => o::BinaryOperator::AddI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Subtract => match base {
                o::DataType::F32 => o::BinaryOperator::SubF,
                o::DataType::I32 => o::BinaryOperator::SubI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Multiply => match base {
                o::DataType::F32 => o::BinaryOperator::MulF,
                o::DataType::I32 => o::BinaryOperator::MulI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Divide => match base {
                o::DataType::F32 => o::BinaryOperator::DivF,
                o::DataType::I32 => o::BinaryOperator::DivI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Modulo => match base {
                o::DataType::F32 => o::BinaryOperator::ModF,
                o::DataType::I32 => o::BinaryOperator::ModI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Power => unimplemented!(),

            i::BinaryOperator::Equal => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::Equal),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::Equal),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::NotEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::NotEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::NotEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::GreaterThan => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThan),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThan),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::GreaterThanOrEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThanOrEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThanOrEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::LessThan => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::LessThan),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::LessThan),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::LessThanOrEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::LessThanOrEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::LessThanOrEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
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

    fn trivialize_assignment(
        &mut self,
        statement: &i::Statement,
        target: &i::VCExpression,
        value: &i::VPExpression,
    ) -> Result<(), CompileProblem> {
        let tvalue = self.trivialize_vp_expression(value.borrow())?;

        let base = self.trivialize_variable(target.base)?;
        let base_type = self.target[base].borrow_type().clone();
        let (new_indexes, _result_type) = self.trivialize_indexes(&target.indexes, base_type)?;
        let base = o::Value::variable(base, &self.target);

        if target.indexes.len() > 0 {
            self.target.add_instruction(o::Instruction::Store {
                from: tvalue,
                to: base,
                to_indexes: new_indexes,
            });
        } else {
            self.target.add_instruction(o::Instruction::Move {
                from: tvalue,
                to: base,
            });
        }
        Ok(())
    }

    fn trivialize_branch(
        &mut self,
        clauses: &Vec<(i::VPExpression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
    ) -> Result<(), CompileProblem> {
        debug_assert!(clauses.len() > 0);
        let end_label = self.target.create_label();
        for (condition_expr, body) in clauses.iter() {
            let condition = self.trivialize_vp_expression(condition_expr)?;
            let body_label = self.target.create_label();
            let next_condition_label = self.target.create_label();
            self.target.add_instruction(o::Instruction::Branch {
                condition,
                true_target: body_label,
                false_target: next_condition_label,
            });
            self.target
                .add_instruction(o::Instruction::Label(body_label));
            for statement in self.source[*body].borrow_body().clone() {
                self.trivialize_statement(&statement)?;
            }
            self.target
                .add_instruction(o::Instruction::Jump { label: end_label });
            self.target
                .add_instruction(o::Instruction::Label(next_condition_label));
        }
        if let Some(body) = /* once told me */ else_clause {
            for statement in self.source[*body].borrow_body().clone() {
                self.trivialize_statement(&statement)?;
            }
        }
        self.target
            .add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_for_loop(
        &mut self,
        counter: i::VariableId,
        start: &i::VPExpression,
        end: &i::VPExpression,
        body: i::ScopeId,
    ) -> Result<(), CompileProblem> {
        let (start_label, end_label) = (self.target.create_label(), self.target.create_label());
        let tcount = o::Value::variable(self.trivialize_variable(counter)?, &self.target);
        let tstart = self.trivialize_vp_expression(start)?;
        let tend = self.trivialize_vp_expression(end)?;
        self.target.add_instruction(o::Instruction::Move {
            from: tstart,
            to: tcount.clone(),
        });
        let condition_var = self
            .target
            .adopt_variable(o::Variable::new(o::DataType::B1));
        let condition_var = o::Value::variable(condition_var, &self.target);
        self.target
            .add_instruction(o::Instruction::BinaryOperation {
                a: tcount.clone(),
                b: tend.clone(),
                x: condition_var.clone(),
                op: o::BinaryOperator::CompI(o::Condition::LessThan),
            });
        self.target.add_instruction(o::Instruction::Branch {
            condition: condition_var.clone(),
            true_target: end_label,
            false_target: start_label,
        });

        self.target
            .add_instruction(o::Instruction::Label(start_label));
        for statement in self.source[body].borrow_body().clone() {
            self.trivialize_statement(&statement)?;
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
            true_target: end_label,
            false_target: start_label,
        });
        self.target
            .add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_indexes(
        &mut self,
        indexes: &Vec<i::VPExpression>,
        data_type: o::DataType,
    ) -> Result<(Vec<o::Value>, o::DataType), CompileProblem> {
        let mut tindexes = Vec::with_capacity(indexes.len());
        let mut element_type = data_type;
        for index in indexes {
            let index_value = self.trivialize_vp_expression(index)?;
            let indext = index_value.get_type(&self.target);
            if indext != o::DataType::I32 {
                // TODO: Check if this is guaranteed to be handled in the previous phase.
                panic!("TODO: Nice error, index is not int scalar.");
            }
            tindexes.push(index_value);
            element_type = if let o::DataType::Array(_, etype) = element_type {
                *etype
            } else {
                unreachable!("Illegal array access should be handled by previous phase.")
            };
        }
        Ok((tindexes, element_type))
    }

    fn trivialize_index(
        &mut self,
        base: &i::VPExpression,
        indexes: &Vec<i::VPExpression>,
    ) -> Result<o::Value, CompileProblem> {
        let base_value = self.trivialize_vp_expression(base)?;
        let (new_indexes, result_type) =
            self.trivialize_indexes(indexes, base_value.get_type(&self.target))?;

        let output_holder = self
            .target
            .adopt_variable(o::Variable::new(result_type.clone()));
        let mut output_value = o::Value::variable(output_holder, &self.target);
        output_value.dimensions = result_type
            .collect_dimensions()
            .iter()
            .map(|dim| (*dim, s::ProxyMode::Keep))
            .collect();
        self.target.add_instruction(o::Instruction::Load {
            from: base_value,
            to: output_value.clone(),
            from_indexes: new_indexes,
        });
        Ok(output_value)
    }

    fn trivialize_vp_expression(
        &mut self,
        expression: &i::VPExpression,
    ) -> Result<o::Value, CompileProblem> {
        Ok(match expression {
            i::VPExpression::Literal(data, ..) => {
                o::Value::literal(Self::trivialize_known_data(data)?)
            }
            i::VPExpression::Variable(id, ..) => {
                o::Value::variable(self.trivialize_variable(*id)?, &self.target)
            }
            i::VPExpression::Index { base, indexes, .. } => self.trivialize_index(base, indexes)?,

            i::VPExpression::UnaryOperation(..) => unimplemented!(),
            i::VPExpression::BinaryOperation {
                lhs, op, rhs, typ, ..
            } => self.trivialize_binary_expression(lhs, *op, rhs, typ)?,

            i::VPExpression::Collect(..) => unimplemented!(),
        })
    }

    fn trivialize_statement(&mut self, statement: &i::Statement) -> Result<(), CompileProblem> {
        Ok(match statement {
            i::Statement::Assert(..) => unimplemented!(),
            i::Statement::Assign { target, value, .. } => {
                self.trivialize_assignment(statement, target, value)?;
            }
            i::Statement::Return(..) => unimplemented!(),
            i::Statement::Branch {
                clauses,
                else_clause,
                ..
            } => {
                self.trivialize_branch(clauses, else_clause)?;
            }
            i::Statement::ForLoop {
                counter,
                start,
                end,
                body,
                ..
            } => {
                self.trivialize_for_loop(*counter, start, end, *body)?;
            }
            i::Statement::MacroCall { mcro, .. } => {
                for statement in self.source[*mcro].borrow_body().clone() {
                    self.trivialize_statement(&statement)?;
                }
            }
        })
    }
}
