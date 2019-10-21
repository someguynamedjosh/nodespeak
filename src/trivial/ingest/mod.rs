use crate::vague::structure as i;
use crate::trivial::structure as o;
use std::collections::HashMap;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut trivializer = Trivializer::new(program);
    trivializer.entry_point();
    trivializer.target
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
            variable_map: HashMap::new()
        }
    }

    fn entry_point(&mut self) {
        unimplemented!()
    }

    fn trivialize_data_type(data_type: &i::DataType) -> (o::VariableType, Vec<u64>) {
        (
            match data_type.borrow_base() {
                i::BaseType::Float => o::VariableType::F32,
                i::BaseType::Int => o::VariableType::I32,
                i::BaseType::Bool => unimplemented!(),
                _ => unreachable!(),
            },
            data_type.borrow_dimensions().iter().map(|expr| match expr {
                i::Expression::Literal(value, ..) => match value {
                    i::KnownData::Int(value) => *value as u64,
                    _ => unreachable!("Resolved array sizes must be integers.")
                },
                _ => unreachable!("Resolved array sizes must be literals.")
            }).collect()
        )
    }

    fn trivialize_variable(&mut self, variable: i::VariableId) -> o::VariableId {
        match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let (var_type, dims) = Self::trivialize_data_type(data_type);
                let trivial_variable = o::Variable::new_array(var_type, dims);
                let id = self.target.adopt_variable(trivial_variable);
                self.variable_map.insert(variable, id);
                id
            }
        }
    }
}