use crate::problem::CompileProblem;
use crate::trivial::structure as o;
use crate::vague::structure as i;
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

    fn trivialize_known_data(data: &i::KnownData) -> o::LiteralData {
        match data {
            i::KnownData::Int(value) => o::LiteralData::Int(*value),
            i::KnownData::Float(value) => o::LiteralData::Float(*value),
            i::KnownData::Bool(value) => o::LiteralData::Bool(*value),
            _ => panic!("TODO: nice error, encountered ct-only data type after simplification.")
        }
    }

    fn trivialize_expression(
        &mut self,
        expression: &i::Expression
    ) -> Result<Option<o::Value>, CompileProblem> {
        Result::Ok(match expression {
            i::Expression::Literal(data, ..) => Option::Some(
                o::Value::Literal(Self::trivialize_known_data(data))
            ),
            _ => unimplemented!("TODO: implement trivialization of {:?}", expression)
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
}
