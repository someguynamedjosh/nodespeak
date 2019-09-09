use crate::problem::FilePosition;
use crate::structure::{Expression, Program, ScopeId, VariableId};
use crate::util::NVec;

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum BaseType {
    Automatic,
    Dynamic(VariableId),
    LoadTemplateParameter(VariableId),
    Bool,
    Int,
    Float,
    Void,
    DataType_,
    Function_,
}

impl BaseType {
    pub fn is_automatic(&self) -> bool {
        match self {
            BaseType::Automatic => true,
            // BaseType::Array(data) => data.base.is_automatic(),
            _ => false,
        }
    }

    pub fn equivalent(&self, other: &BaseType, program: &Program) -> bool {
        match self {
            // If it's a basic type, just check if it is equal to the other one.
            BaseType::Automatic
            | BaseType::Bool
            | BaseType::Int
            | BaseType::Float
            | BaseType::Void
            | BaseType::DataType_
            | BaseType::Function_ => self == other,
            // If it's a dynamic / template type, check the value contained in both targets are both
            // known and identical.
            BaseType::Dynamic(target) | BaseType::LoadTemplateParameter(target) => match other {
                BaseType::Dynamic(other_target) | BaseType::LoadTemplateParameter(other_target) => {
                    let value = program.borrow_temporary_value(*target);
                    let other_value = program.borrow_temporary_value(*other_target);
                    if let KnownData::Unknown = value {
                        false
                    } else {
                        value == other_value
                    }
                }
                _ => false,
            },
        }
    }

    pub fn to_scalar_type(self) -> DataType {
        DataType::scalar(self)
    }

    pub fn to_array_type(self, sizes: Vec<Expression>) -> DataType {
        DataType::array(self, sizes)
    }
}

impl Display for BaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BaseType::Automatic => write!(formatter, "Auto"),
            BaseType::Dynamic(_var) => write!(formatter, "Unresolved"),
            BaseType::LoadTemplateParameter(_var) => write!(formatter, "Unresolved"),
            BaseType::Bool => write!(formatter, "Bool"),
            BaseType::Int => write!(formatter, "Int"),
            BaseType::Float => write!(formatter, "Float"),
            BaseType::Void => write!(formatter, "Void"),
            BaseType::DataType_ => write!(formatter, "DataType_"),
            BaseType::Function_ => write!(formatter, "Function_"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataType {
    base: BaseType,
    sizes: Vec<Expression>,
}

impl DataType {
    pub fn scalar(base: BaseType) -> DataType {
        DataType {
            base,
            sizes: Vec::new(),
        }
    }

    pub fn new(base: BaseType) -> DataType {
        Self::scalar(base)
    }

    pub fn array(base: BaseType, sizes: Vec<Expression>) -> DataType {
        DataType { base, sizes }
    }

    // E.G. if size is 5, [2]Int becomes [5][2]Int.
    pub fn wrap_with_size(&mut self, size: Expression) {
        self.sizes.insert(0, size);
    }

    // E.G. if sizes is 5, 4, 3, then [2][2]Int becomes [5][4][3][2][2]Int.
    pub fn wrap_with_sizes(&mut self, mut sizes: Vec<Expression>) {
        sizes.append(&mut self.sizes);
        self.sizes = sizes;
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_unwraps: usize) -> DataType {
        DataType {
            base: self.base.clone(),
            sizes: Vec::from(&self.sizes[num_unwraps..]),
        }
    }

    pub fn borrow_sizes(&self) -> &Vec<Expression> {
        &self.sizes
    }

    pub fn borrow_base(&self) -> &BaseType {
        &self.base
    }

    pub fn is_scalar(&self) -> bool {
        self.sizes.len() == 0
    }

    pub fn is_array(&self) -> bool {
        !self.is_scalar()
    }

    pub fn is_automatic(&self) -> bool {
        self.base.is_automatic()
    }

    pub fn equivalent(&self, other: &Self, program: &Program) -> bool {
        // Check if the base types are equivalent.
        if !self.base.equivalent(&other.base, program) {
            return false;
        }
        // Check if all the sizes have known and identical values.
        for (size, other_size) in self.sizes.iter().zip(other.sizes.iter()) {
            let size_value = program.borrow_value_of(size);
            let other_size_value = program.borrow_value_of(other_size);
            if let KnownData::Unknown = size_value {
                return false;
            } else if size_value != other_size_value {
                return false;
            }
        }
        true
    }
}

impl Display for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for _ in self.sizes.iter() {
            write!(formatter, "[todo: size]")?;
        }
        write!(formatter, "{}", self.base)
    }
}

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

#[derive(Clone, Debug, PartialEq)]
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
                if contents.borrow_dimensions().len() != data_type.borrow_sizes().len() {
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

impl Display for KnownData {
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
                write!(formatter, "[TODO better format for N-dim arrays.\n")?;
                for value in values.borrow_all_items() {
                    write!(formatter, "\t{},\n", value)?;
                }
                write!(formatter, "]")
            }
            KnownData::DataType(value) => write!(formatter, "{}", value),
            // TODO: Implement function formatter.
            KnownData::Function(_value) => {
                write!(formatter, "[function formatter not implemented]")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    definition: FilePosition,

    data_type: DataType,
    initial_value: KnownData,
    permanent: bool,

    temporary_value: KnownData,
    // temporary_read: bool,
    // multiple_temporary_values: bool,
}

impl Variable {
    fn new_impl(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
        permanent: bool,
    ) -> Variable {
        let initial_value = initial_value.unwrap_or_else(|| KnownData::Unknown);
        Variable {
            definition,
            initial_value: initial_value.clone(),
            data_type,
            permanent,
            temporary_value: initial_value, // temporary_read: false,
                                            // multiple_temporary_values: false,
        }
    }

    pub fn variable(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
    ) -> Variable {
        Self::new_impl(definition, data_type, initial_value, false)
    }

    pub fn constant(definition: FilePosition, data_type: DataType, value: KnownData) -> Variable {
        Self::new_impl(definition, data_type, Option::Some(value), true)
    }

    pub fn function_def(function_data: FunctionData) -> Variable {
        Self::constant(
            function_data.get_header().clone(),
            BaseType::Function_.to_scalar_type(),
            KnownData::Function(function_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::constant(
            definition,
            BaseType::DataType_.to_scalar_type(),
            KnownData::DataType(value),
        )
    }

    pub fn automatic(definition: FilePosition) -> Variable {
        Variable::variable(
            definition,
            BaseType::Automatic.to_scalar_type(),
            Option::None,
        )
    }

    pub fn bool_literal(definition: FilePosition, value: bool) -> Variable {
        Variable::constant(
            definition,
            BaseType::Bool.to_scalar_type(),
            KnownData::Bool(value),
        )
    }

    pub fn int_literal(definition: FilePosition, value: i64) -> Variable {
        Variable::constant(
            definition,
            BaseType::Int.to_scalar_type(),
            KnownData::Int(value),
        )
    }

    pub fn float_literal(definition: FilePosition, value: f64) -> Variable {
        Variable::constant(
            definition,
            BaseType::Float.to_scalar_type(),
            KnownData::Float(value),
        )
    }

    pub fn void(definition: FilePosition) -> Variable {
        Variable::variable(definition, BaseType::Void.to_scalar_type(), Option::None)
    }

    pub fn set_definition(&mut self, new_definition: FilePosition) {
        self.definition = new_definition;
    }

    pub fn get_definition(&self) -> &FilePosition {
        &self.definition
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
    }

    pub fn borrow_data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn set_initial_value(&mut self, value: KnownData) {
        self.initial_value = value;
    }

    pub fn borrow_initial_value(&self) -> &KnownData {
        &self.initial_value
    }

    pub fn mark_as_permanent(&mut self) {
        self.permanent = true;
    }

    pub fn is_permanent(&self) -> bool {
        self.permanent
    }

    pub fn set_temporary_value(&mut self, value: KnownData) {
        self.temporary_value = value;
    }

    pub fn borrow_temporary_value(&self) -> &KnownData {
        &self.temporary_value
    }

    pub fn borrow_temporary_value_mut(&mut self) -> &mut KnownData {
        &mut self.temporary_value
    }

    pub fn reset_temporary_value(&mut self) {
        self.temporary_value = self.initial_value.clone();
    }

    // pub fn set_temporary_value(&mut self, value: KnownData) {
    //     self.temporary_value = value;
    //     if self.temporary_read {
    //         self.multiple_temporary_values = true;
    //     }
    // }

    // pub fn borrow_temporary_value(&mut self) -> &KnownData {
    //     self.temporary_read = true;
    //     &self.temporary_value
    // }

    // pub fn reset_temporary_value(&mut self) {
    //     self.temporary_value = self.initial_value.clone();
    //     self.temporary_read = false;
    //     self.multiple_temporary_values = false;
    // }

    // pub fn had_multiple_temporary_values(&self) -> bool {
    //     self.multiple_temporary_values
    // }
}
