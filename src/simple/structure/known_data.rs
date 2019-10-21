use super::{BaseType, DataType};
use crate::util::NVec;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(NVec<KnownData>)
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Bool(value) => {
                write!(formatter, "{}", if *value { "true" } else { "false" })
            }
            KnownData::Int(value) => write!(formatter, "{}", value),
            KnownData::Float(value) => write!(formatter, "{}", value),
            KnownData::Array(values) => {
                // TODO: put array values on the same line if all the values in the array can fit
                // on the same line.
                let mut indent_level = 0;
                for _ in values.borrow_dimensions() {
                    write!(formatter, "{:indent$}[\n", "", indent = indent_level * 4)?;
                    indent_level += 1;
                }
                let dimensions = values.borrow_dimensions().clone();
                let mut current_index: Vec<_> = dimensions.iter().map(|_| 0).collect();
                'main_loop: loop {
                    write!(
                        formatter,
                        "{:indent$}{:?},\n",
                        "",
                        values.borrow_item(&current_index),
                        indent = indent_level * 4
                    )?;

                    let last_dimension = current_index.len() - 1;
                    current_index[last_dimension] += 1;
                    let mut levels = 0;
                    for dimension in (0..current_index.len()).rev() {
                        if current_index[dimension] < dimensions[dimension] {
                            continue;
                        }
                        levels += 1;
                        if dimension == 0 {
                            break 'main_loop;
                        } else {
                            current_index[dimension] = 0;
                            current_index[dimension - 1] += 1;
                        }
                    }
                    if levels > 0 {
                        for dip in 1..levels {
                            write!(
                                formatter,
                                "{:indent$}]\n",
                                "",
                                indent = (indent_level - dip) * 4
                            )?;
                        }
                        write!(
                            formatter,
                            "{:indent$}],\n{:indent$}[\n",
                            "",
                            "",
                            indent = (indent_level - levels) * 4
                        )?;
                        for dip in (1..levels).rev() {
                            write!(
                                formatter,
                                "{:indent$}[\n",
                                "",
                                indent = (indent_level - dip) * 4
                            )?;
                        }
                    }
                }
                for indent_level in (0..indent_level).rev() {
                    write!(formatter, "{:indent$}]\n", "", indent = indent_level * 4)?;
                }
                write!(formatter, "")
            }
        }
    }
}

impl KnownData {
    pub fn new_array(dimensions: Vec<usize>, base: KnownData) -> KnownData {
        KnownData::Array(NVec::new(dimensions, base))
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        if items.len() == 0 {
            return Self::new_array(vec![], KnownData::Int(0));
        }
        // TODO: Ensure that each data type is compatible.
        if let KnownData::Array(..) = items[0] {
            let mut combined_data = Vec::with_capacity(items.len());
            for item in items {
                if let KnownData::Array(data) = item {
                    combined_data.push(data);
                } else {
                    panic!("TODO nice error, not all data is array.");
                }
            }
            KnownData::Array(NVec::collect(combined_data))
        } else {
            KnownData::Array(NVec::from_vec(items))
        }
    }

    fn get_base_type(&self) -> BaseType {
        match self {
            KnownData::Int(..) => BaseType::Int,
            KnownData::Float(..) => BaseType::Float,
            KnownData::Bool(..) => BaseType::Bool,
            _ => unreachable!(),
        }
    }

    pub fn get_data_type(&self) -> DataType {
        match self {
            KnownData::Array(data) => DataType::array(
                data.borrow_all_items()[0].get_base_type(),
                data.borrow_dimensions()
                    .iter()
                    .map(|dim| {
                        *dim as u64
                    })
                    .collect(),
            ),
            _ => DataType::scalar(self.get_base_type()),
        }
    }

    pub fn require_bool(&self) -> bool {
        match self {
            KnownData::Bool(value) => *value,
            _ => panic!("Expected data to be a bool."),
        }
    }

    pub fn require_int(&self) -> i64 {
        match self {
            KnownData::Int(value) => *value,
            _ => panic!("Expected data to be an int."),
        }
    }

    pub fn require_float(&self) -> f64 {
        match self {
            KnownData::Float(value) => *value,
            _ => panic!("Expected data to be a float."),
        }
    }
}