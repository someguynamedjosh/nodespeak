use std::{
    collections::HashMap,
    ops::{Add, Deref},
    rc::Rc,
};

use itertools::Itertools;
use nom::multi;

use crate::{
    util::{nd_index_iter, NdIndexIter},
    values::{simplify::SimplificationContext, BuiltinOp, BuiltinType, LocalPtr, Value, ValuePtr},
};

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
    Unvectorize(ConcreteValuePtr, u8),
    Vectorize([ConcreteValuePtr; 8]),
    VectorizeByDuplication(ConcreteValuePtr),
    InputScalar { input: usize, position: usize },
    InputVector { input: usize, position: usize },
    UnaryOp(UnaryOp, ConcreteValuePtr),
    BinaryOp(BinaryOp, ConcreteValuePtr, ConcreteValuePtr),
}

impl ConcreteValue {
    pub fn is_vector(&self) -> bool {
        match self {
            Self::Vectorize(..) => true,
            Self::InputVector { .. } => true,
            Self::UnaryOp(_, rhs) => rhs.0.is_vector(),
            Self::BinaryOp(_, lhs, rhs) => lhs.0.is_vector() || rhs.0.is_vector(),
            _ => false,
        }
    }

    pub fn is_scalar(&self) -> bool {
        !self.is_vector()
    }
}

#[derive(Clone, Debug)]
pub struct ConcreteType {
    pub base: ConcreteScalarType,
    pub dims: Vec<usize>,
}

impl ConcreteType {
    /// Expects an index that indexes the outermost (last) dimension first, and
    /// the innermost (first) dimension last. Handles indexes that index into a
    /// broadcasted version of this datatype. E.G. if this type is
    /// Array(Something, 1, 5), you can pass in an index for Array(Something,
    /// 12, 5, 78) and it will still give the correct result.
    pub fn flatten_index(&self, unflattened_index: &[usize]) -> usize {
        assert!(unflattened_index.len() >= self.dims.len());
        let mut index = 0;
        let mut stride = 1;
        for (&dim, &index_part) in self.dims.iter().zip(unflattened_index.iter().rev()) {
            if dim > 1 {
                assert!(index_part < dim);
                index += index_part * stride;
            }
            stride *= dim;
        }
        index
    }

    pub fn size(&self) -> usize {
        self.dims.iter().copied().product()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConcreteScalarType {
    Int,
    Float,
    Bool,
}

impl Add for ConcreteScalarType {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Bool => rhs,
            Self::Int => match rhs {
                Self::Float => Self::Float,
                _ => Self::Int,
            },
            Self::Float => Self::Float,
        }
    }
}

#[derive(Debug)]
pub struct ConcreteProgram {
    inputs: Vec<ConcreteType>,
    outputs: Vec<ConcreteMultiValue>,
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

#[derive(Clone, Debug)]
pub struct ConcreteMultiValue {
    typee: ConcreteType,
    components: Vec<ConcreteValuePtr>,
}

impl ConcreteMultiValue {
    pub fn get_scalar(&self, mut position: usize) -> ConcreteValuePtr {
        for component in &self.components {
            if component.is_scalar() {
                if position == 0 {
                    return component.ptr_clone();
                } else {
                    position -= 1;
                }
            } else {
                if position < 8 {
                    return ConcreteValuePtr::new(ConcreteValue::Unvectorize(
                        component.ptr_clone(),
                        position as _,
                    ));
                } else {
                    position -= 8;
                }
            }
        }
        panic!("Position {} is out of bounds.", position)
    }

    pub fn get_vector(&self, mut position: usize) -> Option<ConcreteValuePtr> {
        for component in &self.components {
            if component.is_scalar() {
                if position == 0 {
                    return None;
                } else {
                    position -= 1;
                }
            } else {
                if position == 0 {
                    return Some(component.ptr_clone());
                } else if position < 8 {
                    return None;
                } else {
                    position -= 8;
                }
            }
        }
        panic!("Position {} is out of bounds.", position)
    }
}

#[derive(Debug)]
pub struct SolidificationContext {
    pub inputs: Vec<(LocalPtr, ConcreteType)>,
    pub converted: HashMap<*const (), ConcreteMultiValue>,
}

fn cast(
    source: ConcreteValuePtr,
    from: ConcreteScalarType,
    to: ConcreteScalarType,
) -> ConcreteValuePtr {
    match (from, to) {
        (ConcreteScalarType::Bool, ConcreteScalarType::Int) => {
            ConcreteValuePtr::new(ConcreteValue::UnaryOp(UnaryOp::BoolToInt, source))
        }
        (ConcreteScalarType::Bool, ConcreteScalarType::Float) => {
            ConcreteValuePtr::new(ConcreteValue::UnaryOp(UnaryOp::BoolToFloat, source))
        }
        (ConcreteScalarType::Int, ConcreteScalarType::Float) => {
            ConcreteValuePtr::new(ConcreteValue::UnaryOp(UnaryOp::IntToFloat, source))
        }
        (from, to) if from == to => source,
        _ => panic!("Invalid cast from {:?} to {:?}", from, to),
    }
}

/// Returns true if the sequence follows a pattern like 5, 6, 7, 8 or -23, -22,
/// -21.
pub fn sequence_monotonically_increases(sequence: &[usize]) -> bool {
    let mut sequence = sequence.iter().copied();
    let mut previous = sequence.next().unwrap();
    for element in sequence {
        if element != previous + 1 {
            return false;
        }
        previous = element;
    }
    true
}

/// Returns true if all elements of the sequence are identical.
pub fn all_identical(sequence: &[usize]) -> bool {
    let mut sequence = sequence.iter().copied();
    let mut previous = sequence.next().unwrap();
    for element in sequence {
        if element != previous {
            return false;
        }
        previous = element;
    }
    true
}

impl SolidificationContext {
    pub fn solidify_type(&mut self, typee: Value) -> ConcreteType {
        let typee = ValuePtr::new(typee);
        typee.check_and_simplify(&mut SimplificationContext::new());
        let typee = typee.borrow().clone();
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

    pub fn solidify_value(&mut self, value: &ValuePtr) -> ConcreteMultiValue {
        if let Some(existing) = self.converted.get(&value.as_ptr()) {
            return existing.clone();
        }
        let multi_value = match &*value.borrow() {
            Value::BuiltinType(_) => panic!("Types are not available at runtime."),
            Value::BuiltinOp(_) => panic!("Functions are not available at runtime."),
            Value::Noop => panic!("Tried to take the value of a noop."),
            Value::Malformed => panic!("Tried to take the value of a malformed expression."),
            &Value::FloatLiteral(value) => ConcreteMultiValue {
                typee: ConcreteType {
                    base: ConcreteScalarType::Float,
                    dims: vec![],
                },
                components: vec![ConcreteValuePtr::new(ConcreteValue::FloatLiteral(value))],
            },
            &Value::IntLiteral(value) => ConcreteMultiValue {
                typee: ConcreteType {
                    base: ConcreteScalarType::Int,
                    dims: vec![],
                },
                components: vec![ConcreteValuePtr::new(ConcreteValue::IntLiteral(value))],
            },
            &Value::BoolLiteral(value) => ConcreteMultiValue {
                typee: ConcreteType {
                    base: ConcreteScalarType::Bool,
                    dims: vec![],
                },
                components: vec![ConcreteValuePtr::new(ConcreteValue::BoolLiteral(value))],
            },
            Value::ArrayLiteral { elements, dims } => {
                let mut new_dims = Vec::new();
                for dim in dims {
                    if let &Value::IntLiteral(dim) = &*dim.borrow() {
                        if dim < 1 {
                            panic!("Dim must be greater than zero.");
                        }
                        new_dims.push(dim as usize);
                    } else {
                        panic!("Array dimension must be determinable at compile time.");
                    }
                }
                let mut components = Vec::new();
                let num_elements = elements.len();
                let first_nonvectorized_element = num_elements - num_elements % 8;
                let mut element_type = ConcreteScalarType::Bool;
                for element in elements {
                    element_type = element_type + self.solidify_value(element).typee.base;
                }
                let mut convert_and_expect_scalar = |element: &ValuePtr| {
                    let ConcreteMultiValue { typee, components } = self.solidify_value(element);
                    assert!(
                        components.len() == 1,
                        "Array literals must be composed of scalars."
                    );
                    let component = components.into_iter().next().unwrap();
                    assert!(
                        component.is_scalar(),
                        "Array literals must be composed of scalars."
                    );
                    cast(component, typee.base, element_type)
                };
                for idx in 0..first_nonvectorized_element / 8 {
                    let scalars = [
                        convert_and_expect_scalar(&elements[8 * idx + 0]),
                        convert_and_expect_scalar(&elements[8 * idx + 1]),
                        convert_and_expect_scalar(&elements[8 * idx + 2]),
                        convert_and_expect_scalar(&elements[8 * idx + 3]),
                        convert_and_expect_scalar(&elements[8 * idx + 4]),
                        convert_and_expect_scalar(&elements[8 * idx + 5]),
                        convert_and_expect_scalar(&elements[8 * idx + 6]),
                        convert_and_expect_scalar(&elements[8 * idx + 7]),
                    ];
                    components.push(ConcreteValuePtr::new(ConcreteValue::Vectorize(scalars)));
                }
                for idx in first_nonvectorized_element..elements.len() {
                    components.push(convert_and_expect_scalar(&elements[idx]));
                }
                ConcreteMultiValue {
                    typee: ConcreteType {
                        base: element_type,
                        dims: new_dims,
                    },
                    components,
                }
            }
            Value::Declaration(_) => panic!("Tried to take the value of a declaration."),
            Value::Local(local) => {
                let mut result = None;
                for (input_index, input) in self.inputs.iter().enumerate() {
                    if &input.0 == local {
                        let mut components = Vec::new();
                        let vector_components = input.1.size() / 8;
                        let scalar_components = input.1.size() % 8;
                        for position in 0..vector_components {
                            components.push(ConcreteValuePtr::new(ConcreteValue::InputVector {
                                input: input_index,
                                position: position * 8,
                            }));
                        }
                        for position in 0..scalar_components {
                            components.push(ConcreteValuePtr::new(ConcreteValue::InputScalar {
                                input: input_index,
                                position: vector_components * 8 + position,
                            }));
                        }
                        result = Some(ConcreteMultiValue {
                            components,
                            typee: input.1.clone(),
                        })
                    }
                }
                result.expect("Tried to take the value of a local that isn't an input.")
            }
            Value::Assignment { .. } => {
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
                        let mut components = Vec::new();
                        for component in rhs.components {
                            components.push(cast(component, rhs_type.base, new_type.base));
                        }
                        ConcreteMultiValue {
                            components,
                            // Specifically use the dims of the rhs because we don't broadcast those
                            // until we encounter an operation that requires it.
                            typee: ConcreteType {
                                base: new_type.base,
                                dims: rhs_type.dims,
                            },
                        }
                    } else {
                        let lhs = self.solidify_value(&args[0]);
                        let result_type = self.solidify_type(value.typee());
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
                        let lhs_indexes =
                            nd_index_iter(result_type.dims.iter().copied().rev().collect())
                                .map(|idx| lhs.typee.flatten_index(&idx))
                                .collect_vec();
                        let rhs_indexes =
                            nd_index_iter(result_type.dims.iter().copied().rev().collect())
                                .map(|idx| rhs.typee.flatten_index(&idx))
                                .collect_vec();
                        assert_eq!(lhs_indexes.len(), rhs_indexes.len());
                        let mut next_index = 0;
                        let mut components = Vec::new();
                        while next_index < lhs_indexes.len() {
                            if lhs_indexes.len() - next_index >= 8 {
                                let indexes_in_question = next_index..next_index + 8;
                                let lhs_vector = if sequence_monotonically_increases(
                                    &lhs_indexes[indexes_in_question.clone()],
                                ) {
                                    lhs.get_vector(lhs_indexes[next_index])
                                } else if all_identical(&lhs_indexes[indexes_in_question.clone()]) {
                                    Some(ConcreteValuePtr::new(
                                        ConcreteValue::VectorizeByDuplication(
                                            lhs.get_scalar(lhs_indexes[next_index]),
                                        ),
                                    ))
                                } else {
                                    None
                                };
                                let rhs_vector = if sequence_monotonically_increases(
                                    &rhs_indexes[indexes_in_question.clone()],
                                ) {
                                    rhs.get_vector(rhs_indexes[next_index])
                                } else if all_identical(&rhs_indexes[indexes_in_question.clone()]) {
                                    Some(ConcreteValuePtr::new(
                                        ConcreteValue::VectorizeByDuplication(
                                            rhs.get_scalar(rhs_indexes[next_index]),
                                        ),
                                    ))
                                } else {
                                    None
                                };
                                if let (Some(lhs_vector), Some(rhs_vector)) =
                                    (lhs_vector, rhs_vector)
                                {
                                    components.push(ConcreteValuePtr::new(
                                        ConcreteValue::BinaryOp(op, lhs_vector, rhs_vector),
                                    ));
                                    next_index += 8;
                                    continue;
                                }
                            }
                            components.push(ConcreteValuePtr::new(ConcreteValue::BinaryOp(
                                op,
                                lhs.get_scalar(next_index),
                                rhs.get_scalar(next_index),
                            )))
                        }
                        println!("{:#?} {:#?}", lhs_indexes, rhs_indexes);
                        ConcreteMultiValue {
                            components,
                            typee: result_type,
                        }
                    }
                } else {
                    todo!()
                }
            }
        };
        self.converted.insert(value.as_ptr(), multi_value.clone());
        multi_value
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
