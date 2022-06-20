use std::{
    cell::{Ref, RefMut},
    fmt::{self, Debug, Formatter},
    ops::{Add, BitAnd, BitOr, Deref, Div, Mul, Not, Rem, Sub},
    rc::Rc,
};

use super::{BuiltinOp, BuiltinType, Value};
use crate::util::{rcrc, Rcrc};

#[derive(Clone, PartialEq)]
pub struct ValuePtr(pub(super) Rcrc<Value>);

impl Debug for ValuePtr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.borrow().fmt(f)
    }
}

impl ValuePtr {
    pub fn new(value: Value) -> Self {
        Self(rcrc(value))
    }

    pub fn ptr_clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    pub fn borrow(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.0.borrow_mut()
    }

    pub fn as_ptr(&self) -> *const () {
        Rc::as_ptr(&self.0).cast()
    }

    pub(crate) fn as_type(&self) -> Option<BuiltinType> {
        if let Value::BuiltinType(typee) = &*self.borrow() {
            Some(typee.clone())
        } else {
            None
        }
    }

    pub fn deep_clone(&self) -> Self {
        Self::new(self.0.borrow().deep_clone())
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
                Self::new(Value::FunctionCall(
                    ValuePtr::new(Value::BuiltinOp(BuiltinOp::$EnumVariant)),
                    vec![self, rhs],
                    0,
                ))
            }
        }
    };
}

macro_rules! impl_op_without_trait {
    ($EnumVariant:ident, $fn_name:ident) => {
        impl ValuePtr {
            pub fn $fn_name(&self, rhs: &Self) -> Self {
                Self::new(Value::FunctionCall(
                    ValuePtr::new(Value::BuiltinOp(BuiltinOp::$EnumVariant)),
                    vec![self.ptr_clone(), rhs.ptr_clone()],
                    0,
                ))
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
        Self::new(Value::FunctionCall(
            ValuePtr::new(Value::BuiltinOp(BuiltinOp::Not)),
            vec![self],
            0,
        ))
    }
}

impl_op_without_trait!(Gt, gt);
impl_op_without_trait!(Gte, gte);
impl_op_without_trait!(Lt, lt);
impl_op_without_trait!(Lte, lte);
impl_op_without_trait!(Eq, eq);
impl_op_without_trait!(Neq, neq);
