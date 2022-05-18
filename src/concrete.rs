use std::{collections::HashMap, ops::Deref, rc::Rc};

use crate::values::{BuiltinOp, BuiltinType, LocalPtr, Value, ValuePtr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisBroadcastBehavior {
    /// Treat a n-length dimension as an n-length dimension.
    Keep,
    /// Broadcast a 1-length dimension to a larger size.
    Stretch,
    /// Treat the array as having an extra 1-length dimension added to it
    /// (stretch it.)
    Create,
}

pub type BroadcastBehavior = Vec<AxisBroadcastBehavior>;

#[derive(Clone, Debug)]
pub enum ConcreteValue {
    IntLiteral(i32),
    FloatLiteral(f32),
    BoolLiteral(bool),
    ArrayLiteral {
        elements: Vec<ConcreteValuePtr>,
        dims: Vec<usize>,
    },
    Input(usize),
    UnaryOp(UnaryOp, ConcreteValuePtr),
    BinaryOp(BinaryOp, ConcreteValuePtr, ConcreteValuePtr),
}

#[derive(Clone, Debug)]
pub struct ConcreteType {
    pub base: ConcreteScalarType,
    pub dims: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConcreteScalarType {
    Int,
    Float,
    Bool,
}

#[derive(Debug)]
pub struct ConcreteProgram {
    inputs: Vec<ConcreteType>,
    outputs: Vec<ConcreteValuePtr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    IntToFloat,
    BoolToInt,
    BoolToFloat,
    Not,
    Noop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug)]
pub struct ConcreteValuePtr(Rc<ConcreteValue>);

impl ConcreteValuePtr {
    pub fn new(value: ConcreteValue) -> Self {
        Self(Rc::new(value))
    }

    pub fn ptr_clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Deref for ConcreteValuePtr {
    type Target = ConcreteValue;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Debug)]
pub struct SolidificationContext {
    pub inputs: Vec<(LocalPtr, ConcreteType)>,
    pub converted: HashMap<*const (), ConcreteValuePtr>,
}

impl SolidificationContext {
    pub fn solidify_type(&mut self, typee: Value) -> ConcreteType {
        match typee {
            Value::BuiltinType(typee) => match typee {
                BuiltinType::Int => ConcreteType {
                    base: ConcreteScalarType::Int,
                    dims: vec![],
                },
                BuiltinType::Float => ConcreteType {
                    base: ConcreteScalarType::Float,
                    dims: vec![],
                },
                BuiltinType::Bool => ConcreteType {
                    base: ConcreteScalarType::Bool,
                    dims: vec![],
                },
                BuiltinType::Any => panic!("Any types are not available at runtime."),
                BuiltinType::Type => panic!("Not available at runtime."),
                BuiltinType::Array { eltype, dims } => {
                    let mut base = self.solidify_type(eltype.borrow().clone());
                    for dim in dims {
                        if let &Value::IntLiteral(dim) = &*dim.borrow() {
                            if dim > 0 {
                                base.dims.push(dim as usize);
                            } else {
                                panic!("Array dimension must be greater than zero.");
                            }
                        } else {
                            panic!("Array dimension must be an integer.");
                        }
                    }
                    base
                }
                BuiltinType::InSet { eltype, .. } => self.solidify_type(eltype.borrow().clone()),
                BuiltinType::Function { .. } => panic!("Not available at runtime."),
                BuiltinType::Malformed => panic!("Invalid operation."),
            },
            _ => panic!("{:#?} is not a type", typee),
        }
    }

    pub fn solidify_value(&mut self, value: &ValuePtr) -> ConcreteValuePtr {
        if let Some(existing) = self.converted.get(&value.as_ptr()) {
            return existing.ptr_clone();
        }
        let concrete_value = match &*value.borrow() {
            Value::BuiltinType(_) => panic!("Types are not available at runtime."),
            Value::BuiltinOp(_) => panic!("Functions are not available at runtime."),
            Value::Noop => panic!("Tried to take the value of a noop."),
            Value::Malformed => panic!("Tried to take the value of a malformed expression."),
            &Value::FloatLiteral(value) => ConcreteValue::FloatLiteral(value),
            &Value::IntLiteral(value) => ConcreteValue::IntLiteral(value),
            &Value::BoolLiteral(value) => ConcreteValue::BoolLiteral(value),
            Value::ArrayLiteral { elements, dims } => {
                let mut new_dims = Vec::new();
                for dim in dims {
                    if let &Value::IntLiteral(dim) = &*dim.borrow() {
                        if dim < 1 {
                            panic!("Dim must be greater than zero.");
                        }
                        new_dims.push(dim as usize);
                    }
                }
                ConcreteValue::ArrayLiteral {
                    elements: elements.iter().map(|x| self.solidify_value(x)).collect(),
                    dims: new_dims,
                }
            }
            Value::Declaration(_) => panic!("Tried to take the value of a declaration."),
            Value::Local(local) => {
                let mut result = None;
                for (index, input) in self.inputs.iter().enumerate() {
                    if &input.0 == local {
                        result = Some(ConcreteValue::Input(index))
                    }
                }
                result.expect("Tried to take the value of a local that isn't an input.")
            }
            Value::Assignment { base, target } => {
                panic!("Tried to take the value of an assignment.")
            }
            Value::Function { .. } => panic!("Functions are not available at runtime."),
            Value::FunctionCall(base, args, result) => {
                assert_eq!(result, &0);
                if args.len() == 2 {
                    let rhs = self.solidify_value(&args[1]);
                    if &*base.borrow() == &Value::BuiltinOp(BuiltinOp::Cast) {
                        let new_type = self.solidify_type(args[0].borrow().clone());
                        let rhs_type = self.solidify_type(args[1].typee());
                        let op = if rhs_type.base == new_type.base {
                            UnaryOp::Noop
                        } else {
                            match (rhs_type.base, new_type.base) {
                                (ConcreteScalarType::Bool, ConcreteScalarType::Int) => {
                                    UnaryOp::BoolToInt
                                }
                                (ConcreteScalarType::Bool, ConcreteScalarType::Float) => {
                                    UnaryOp::BoolToFloat
                                }
                                (ConcreteScalarType::Int, ConcreteScalarType::Float) => {
                                    UnaryOp::IntToFloat
                                }
                                _ => panic!("Invalid cast from {:?} to {:?}", rhs_type, new_type),
                            }
                        };
                        ConcreteValue::UnaryOp(op, rhs)
                    } else {
                        let lhs = self.solidify_value(&args[0]);
                        let op = match &*base.borrow() {
                            Value::BuiltinOp(BuiltinOp::Add) => BinaryOp::Add,
                            Value::BuiltinOp(BuiltinOp::Sub) => BinaryOp::Sub,
                            Value::BuiltinOp(BuiltinOp::Mul) => BinaryOp::Mul,
                            Value::BuiltinOp(BuiltinOp::Div) => BinaryOp::Div,
                            Value::BuiltinOp(BuiltinOp::Rem) => BinaryOp::Rem,
                            Value::BuiltinOp(BuiltinOp::Gt) => BinaryOp::Gt,
                            Value::BuiltinOp(BuiltinOp::Lt) => BinaryOp::Lt,
                            Value::BuiltinOp(BuiltinOp::Gte) => BinaryOp::Gte,
                            Value::BuiltinOp(BuiltinOp::Lte) => BinaryOp::Lte,
                            Value::BuiltinOp(BuiltinOp::Eq) => BinaryOp::Eq,
                            Value::BuiltinOp(BuiltinOp::Neq) => BinaryOp::Neq,
                            Value::BuiltinOp(BuiltinOp::And) => BinaryOp::And,
                            Value::BuiltinOp(BuiltinOp::Or) => BinaryOp::Or,
                            Value::BuiltinOp(BuiltinOp::Xor) => BinaryOp::Xor,
                            _ => panic!("Invalid binary operation."),
                        };
                        ConcreteValue::BinaryOp(op, lhs, rhs)
                    }
                } else {
                    todo!()
                }
            }
        };
        let concrete_value = ConcreteValuePtr::new(concrete_value);
        self.converted
            .insert(value.as_ptr(), concrete_value.ptr_clone());
        concrete_value
    }
}

pub fn solidify(function: ValuePtr) -> ConcreteProgram {
    if let Value::Function {
        inputs,
        outputs: liquid_outputs,
        body,
        ..
    } = &*function.borrow()
    {
        let mut ctx = SolidificationContext {
            inputs: Vec::new(),
            converted: HashMap::new(),
        };
        for input in inputs {
            let typee = ctx.solidify_type(input.typee.borrow().clone());
            ctx.inputs.push((input.ptr_clone(), typee))
        }
        let mut outputs = Vec::new();
        'next_output: for output in liquid_outputs {
            for statement in body {
                if let Value::Assignment { base, target } = &*statement.borrow() {
                    if target == output {
                        outputs.push(ctx.solidify_value(base));
                        continue 'next_output;
                    }
                }
            }
            panic!("Output not assigned a value!");
        }
        ConcreteProgram {
            inputs: ctx.inputs.into_iter().map(|x| x.1).collect(),
            outputs,
        }
    } else {
        panic!("Not a function.")
    }
}
