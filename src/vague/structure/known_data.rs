use super::{BaseType, DataType};
use crate::problem::FilePosition;
use crate::util::NVec;
use crate::vague::structure::{Expression, ScopeId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Debug)]
pub struct FunctionData {
    body: ScopeId,
    header: FilePosition,
}

impl PartialEq for FunctionData {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl FunctionData {
    pub fn new(body: ScopeId, header: FilePosition) -> FunctionData {
        FunctionData { body, header }
    }

    pub fn set_header(&mut self, new_header: FilePosition) {
        self.header = new_header;
    }

    pub fn get_header(&self) -> &FilePosition {
        &self.header
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }
}

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Void,
    Unknown,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(DataType),
    Function(FunctionData),
    Array(NVec<KnownData>),
}

impl KnownData {
    pub fn new_array(dimensions: Vec<usize>) -> KnownData {
        KnownData::Array(NVec::new(dimensions, KnownData::Unknown))
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        if items.len() == 0 {
            return KnownData::new_array(vec![]);
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
            KnownData::Void => BaseType::Void,
            KnownData::Bool(..) => BaseType::Bool,
            KnownData::Int(..) => BaseType::Int,
            KnownData::Float(..) => BaseType::Float,
            KnownData::DataType(..) => BaseType::DataType_,
            KnownData::Function(..) => BaseType::Function_,
            _ => unreachable!(),
        }
    }

    /// Returns Err if the value is KnownData::Unknown.
    pub fn get_data_type(&self) -> Result<DataType, ()> {
        Result::Ok(match self {
            KnownData::Unknown => return Result::Err(()),
            KnownData::Array(data) => DataType::array(
                data.borrow_all_items()[0].get_base_type(),
                data.borrow_dimensions()
                    .iter()
                    .map(|dim| {
                        Expression::Literal(
                            KnownData::Int(*dim as i64),
                            FilePosition::placeholder(),
                        )
                    })
                    .collect(),
            ),
            _ => DataType::scalar(self.get_base_type()),
        })
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

    pub fn require_data_type(&self) -> &DataType {
        match self {
            KnownData::DataType(value) => value,
            _ => panic!("Expected data to be a data type."),
        }
    }

    pub fn require_function(&self) -> &FunctionData {
        match self {
            KnownData::Function(value) => value,
            _ => panic!("Expected data to be a function."),
        }
    }

    pub fn require_array(&self) -> &NVec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn require_array_mut(&mut self) -> &mut NVec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    fn matches_base_type(&self, base_type: &BaseType) -> bool {
        match self {
            KnownData::Void => {
                if let BaseType::Void = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Unknown => false,
            KnownData::Bool(_value) => {
                if let BaseType::Bool = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Int(_value) => {
                if let BaseType::Int = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Float(_value) => {
                if let BaseType::Float = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::DataType(_value) => {
                if let BaseType::DataType_ = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Function(_value) => {
                if let BaseType::Function_ = base_type {
                    true
                } else {
                    false
                }
            }
            _ => panic!("No other base types exist."),
        }
    }

    pub fn matches_data_type(&self, data_type: &DataType) -> bool {
        match self {
            KnownData::Array(contents) => {
                // Check that the data has the same dimensions as the data type.
                if contents.borrow_dimensions().len() != data_type.borrow_dimensions().len() {
                    return false;
                }
                // TODO? check dimensions
                // Check that all the elements have the correct type.
                for item in contents.borrow_all_items() {
                    if !item.matches_base_type(data_type.borrow_base()) {
                        return false;
                    }
                }
                true
            }
            _ => self.matches_base_type(data_type.borrow_base()),
        }
    }
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Void => write!(formatter, "[void]"),
            KnownData::Unknown => write!(formatter, "[unknown]"),
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
            KnownData::DataType(value) => write!(formatter, "{:?}", value),
            // TODO: Implement function formatter.
            KnownData::Function(value) => {
                write!(formatter, "function with body at {:?}", value.body)
            }
        }
    }
}
