use std::fmt::{self, Debug, Formatter};

use crate::shared::{NativeBaseType, NativeType, ProxyMode};
use crate::util::{self, NVec};

#[derive(Clone, PartialEq)]
pub enum NativeData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(NVec<NativeData>),
}

impl Debug for NativeData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "{}i32", value),
            Self::Float(value) => write!(formatter, "{}f32", value),
            Self::Bool(value) => write!(formatter, "{}b8", if *value { "true" } else { "false" }),
            Self::Array(values) => {
                let dims = values.borrow_dimensions();
                assert!(dims.len() > 0);
                for index in util::nd_index_iter(Vec::from(dims)) {
                    for component in index.iter().rev() {
                        if *component == 0 {
                            write!(formatter, "[")?;
                        } else {
                            break;
                        }
                    }
                    write!(formatter, "{:?}", values.borrow_item(&index[..]))?;
                    for (dim_index, component) in index.iter().enumerate().rev() {
                        if *component == dims[dim_index] - 1 {
                            write!(formatter, "]")?;
                        } else {
                            break;
                        }
                    }
                    write!(formatter, ", ")?;
                }
                write!(formatter, "")
            }
        }
    }
}

impl NativeData {
    pub fn from_binary_data(target_type: &NativeType, data: &[u8]) -> NativeData {
        assert!(target_type.get_physical_size() == data.len());
        if target_type.borrow_dimensions().len() > 0 {
            let dims: Vec<_> = target_type
                .borrow_dimensions()
                .iter()
                .map(|(dim, _)| *dim)
                .collect();
            let num_items = dims.iter().fold(1, |num, dim| num * dim);
            let item_type = target_type.clone_and_unwrap(dims.len());
            let item_size = item_type.get_physical_size();
            let mut items = Vec::with_capacity(num_items);
            for index in 0..num_items {
                items.push(NativeData::from_binary_data(
                    &item_type,
                    &data[index * item_size..(index + 1) * item_size],
                ));
            }
            NativeData::Array(NVec::from_vec_and_dims(items, dims))
        } else {
            match target_type.get_base() {
                NativeBaseType::B8 => Self::Bool(data[0] > 0),
                NativeBaseType::I32 => {
                    Self::Int(i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64)
                }
                NativeBaseType::F32 => {
                    Self::Float(f32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64)
                }
            }
        }
    }

    pub fn get_type(&self) -> NativeType {
        match self {
            Self::Int(..) => NativeType::int_scalar(),
            Self::Float(..) => NativeType::float_scalar(),
            Self::Bool(..) => NativeType::bool_scalar(),
            Self::Array(nvec) => {
                let items = nvec.borrow_all_items();
                assert!(items.len() > 0);
                let base_type = items[0].get_type();
                let dimensions = nvec.borrow_dimensions().clone();
                NativeType::array(
                    base_type.get_base(),
                    dimensions
                        .iter()
                        .map(|index| (*index, ProxyMode::Keep))
                        .collect(),
                )
            }
        }
    }

    pub fn binary_data(&self) -> u32 {
        match self {
            Self::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            Self::Int(value) => *value as i32 as u32,
            Self::Float(value) => f32::to_bits(*value as f32),
            Self::Array(..) => unimplemented!(),
        }
    }

    fn add_binary_data(&self, to: &mut Vec<u8>) {
        match self {
            Self::Bool(value) => to.push(if *value { 1 } else { 0 }),
            Self::Int(value) => {
                for byte in (*value as i32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Float(value) => {
                for byte in (*value as f32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Array(values) => {
                for value in values.borrow_all_items().iter() {
                    value.add_binary_data(to);
                }
            }
        }
    }

    pub fn arbitrary_len_binary_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        self.add_binary_data(&mut data);
        data
    }

    pub fn require_int(&self) -> i64 {
        if let Self::Int(value) = self {
            *value
        } else {
            panic!("Required an Int, but got a {:?}.", self)
        }
    }

    pub fn require_float(&self) -> f64 {
        if let Self::Float(value) = self {
            *value
        } else {
            panic!("Required an Float, but got a {:?}.", self)
        }
    }
}