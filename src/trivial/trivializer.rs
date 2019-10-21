use crate::structure as s;
use crate::trivial as t;
use std::collections::HashMap;

pub fn trivialize(program: &s::Program) -> t::Program {
    let mut trivializer = Trivializer::new(program);
    trivializer.entry_point();
    trivializer.target
}

struct Trivializer<'a> {
    source: &'a s::Program,
    target: t::Program,
    variable_map: HashMap<s::VariableId, t::VariableId>,
}

impl<'a> Trivializer<'a> {
    fn new(source: &s::Program) -> Trivializer {
        Trivializer {
            source,
            target: t::Program::new(),
            variable_map: HashMap::new()
        }
    }

    fn entry_point(&mut self) {
        unimplemented!()
    }

    fn trivialize_data_type(data_type: &s::DataType) -> (t::VariableType, Vec<u64>) {
        (
            match data_type.borrow_base() {
                s::BaseType::Float => t::VariableType::F32,
                s::BaseType::Int => t::VariableType::I32,
                s::BaseType::Bool => unimplemented!(),
                _ => unreachable!(),
            },
            data_type.borrow_dimensions().iter().map(|expr| match expr {
                s::Expression::Literal(value, ..) => match value {
                    s::KnownData::Int(value) => *value as u64,
                    _ => unreachable!("Resolved array sizes must be integers.")
                },
                _ => unreachable!("Resolved array sizes must be literals.")
            }).collect()
        )
    }

    fn trivialize_variable(&mut self, variable: s::VariableId) -> t::VariableId {
        match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let (var_type, dims) = Self::trivialize_data_type(data_type);
                let trivial_variable = t::Variable::new_array(var_type, dims);
                let id = self.target.adopt_variable(trivial_variable);
                self.variable_map.insert(variable, id);
                id
            }
        }
    }
}