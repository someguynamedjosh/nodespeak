use std::{
    ops::{Add, BitAnd, BitOr, Deref, Div, Mul, Not, Rem, Sub},
    rc::Rc,
};

use super::{BuiltinOp, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct ValuePtr(pub(super) Rc<Value>);

impl ValuePtr {
    pub fn new(value: Value) -> Self {
        Self(Rc::new(value))
    }

    pub fn ptr_clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Deref for ValuePtr {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

macro_rules! impl_op {
    ($TraitName:ident, $fn_name:ident) => {
        impl_op!($TraitName, $fn_name, $TraitName);
    };
    ($TraitName:ident, $fn_name:ident, $EnumVariant:ident) => {
        impl $TraitName for ValuePtr {
            type Output = Self;

            fn $fn_name(self, rhs: Self) -> Self {
                Self(Rc::new(Value::FunctionCall(
                    ValuePtr::new(Value::BuiltinOp(BuiltinOp::$EnumVariant)),
                    vec![self, rhs],
                    0,
                )))
            }
        }
    };
}

macro_rules! impl_op_without_trait {
    ($EnumVariant:ident, $fn_name:ident) => {
        impl ValuePtr {
            pub fn $fn_name(&self, rhs: &Self) -> Self {
                Self(Rc::new(Value::FunctionCall(
                    ValuePtr::new(Value::BuiltinOp(BuiltinOp::$EnumVariant)),
                    vec![self.ptr_clone(), rhs.ptr_clone()],
                    0,
                )))
            }
        }
    };
}

impl_op!(Add, add, Add);
impl_op!(Sub, sub, Sub);
impl_op!(Mul, mul, Mul);
impl_op!(Div, div, Div);
impl_op!(Rem, rem, Rem);
impl_op!(BitAnd, bitand, And);
impl_op!(BitOr, bitor, Or);

impl Not for ValuePtr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(Rc::new(Value::FunctionCall(
            ValuePtr::new(Value::BuiltinOp(BuiltinOp::Not)),
            vec![self],
            0,
        )))
    }
}

impl_op_without_trait!(Gt, gt);
impl_op_without_trait!(Gte, gte);
impl_op_without_trait!(Lt, lt);
impl_op_without_trait!(Lte, lte);
impl_op_without_trait!(Eq, eq);
impl_op_without_trait!(Neq, neq);
