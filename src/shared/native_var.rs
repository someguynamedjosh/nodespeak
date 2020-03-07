use std::fmt::{self, Debug, Formatter};
use crate::shared::ProxyMode;

#[derive(Clone, Copy, PartialEq)]
pub enum NativeBaseType {
    F32,
    I32,
    B8,
}

impl NativeBaseType {
    pub fn get_physical_size(&self) -> usize {
        match self {
            NativeBaseType::F32 | NativeBaseType::I32 => 4,
            NativeBaseType::B8 => 1,
        }
    }
}

impl Debug for NativeBaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::F32 => "f32",
                Self::I32 => "i32",
                Self::B8 => "b8",
            }
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct NativeType {
    base: NativeBaseType,
    dimensions: Vec<(u64, ProxyMode)>,
}

impl Debug for NativeType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for dim in self.dimensions.iter() {
            // TODO: output proxy mode.
            write!(formatter, "[{}]", dim.0)?;
        }
        write!(formatter, "{:?}", self.base)
    }
}

impl NativeType {
    pub fn array(base: NativeBaseType, dimensions: Vec<(u64, ProxyMode)>) -> NativeType {
        NativeType { base, dimensions }
    }

    pub fn scalar(base: NativeBaseType) -> NativeType {
        Self::array(base, vec![])
    }

    pub fn int_scalar() -> NativeType {
        Self::scalar(NativeBaseType::I32)
    }

    pub fn float_scalar() -> NativeType {
        Self::scalar(NativeBaseType::F32)
    }

    pub fn bool_scalar() -> NativeType {
        Self::scalar(NativeBaseType::B8)
    }

    pub fn get_base(&self) -> NativeBaseType {
        self.base
    }

    pub fn borrow_dimensions(&self) -> &Vec<(u64, ProxyMode)> {
        &self.dimensions
    }

    pub fn is_int(&self) -> bool {
        self.base == NativeBaseType::I32
    }

    pub fn is_float(&self) -> bool {
        self.base == NativeBaseType::F32
    }

    pub fn is_scalar(&self) -> bool {
        self.dimensions.len() == 0
    }

    pub fn is_array(&self) -> bool {
        self.dimensions.len() > 0
    }

    pub fn get_physical_size(&self) -> u64 {
        let mut size = self.base.get_physical_size() as u64;
        for dim in &self.dimensions {
            size *= dim.0;
        }
        size
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_dimensions: usize) -> Self {
        Self {
            base: self.base.clone(),
            dimensions: Vec::from(&self.dimensions[num_dimensions..]),
        }
    }
}

#[derive(Clone)]
pub struct NativeVar {
    typ: NativeType,
}

impl Debug for NativeVar {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.typ)
    }
}

impl NativeVar {
    pub fn new(typ: NativeType) -> NativeVar {
        NativeVar { typ }
    }

    pub fn borrow_type(&self) -> &NativeType {
        &self.typ
    }

    pub fn borrow_dimensions(&self) -> &Vec<(u64, ProxyMode)> {
        self.typ.borrow_dimensions()
    }

    pub fn get_physical_size(&self) -> u64 {
        self.typ.get_physical_size()
    }
}
