use crate::llvmir::structure as o;
use crate::shared::{self, ProxyMode};
use crate::trivial::structure as i;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::transforms as llvmt;
use llvm_sys::*;
use std::collections::{HashMap, HashSet};

const UNNAMED: *const libc::c_char = b"\0".as_ptr() as *const libc::c_char;

struct Intrinsics {
    sqrt_f32: LLVMValueRef,
    powi_i32: LLVMValueRef,
    sin_f32: LLVMValueRef,
    cos_f32: LLVMValueRef,
    pow_f32: LLVMValueRef,
    exp_f32: LLVMValueRef,
    exp2_f32: LLVMValueRef,
    log_f32: LLVMValueRef,
    log10_f32: LLVMValueRef,
    log2_f32: LLVMValueRef,
    fabs_f32: LLVMValueRef,
    floor_f32: LLVMValueRef,
    ceil_f32: LLVMValueRef,
    trunc_f32: LLVMValueRef,
}

impl Intrinsics {
    fn new(module: LLVMModuleRef, context: LLVMContextRef) -> Self {
        let make = |name_nullterm: &[u8]| -> LLVMValueRef {
            unsafe {
                let float_type = LLVMFloatTypeInContext(context);
                let mut arg_types = [float_type];
                let fn_type = LLVMFunctionType(float_type, arg_types.as_mut_ptr(), 1, 0);
                LLVMAddFunction(module, name_nullterm.as_ptr() as *const _, fn_type)
            }
        };
        let make_f32_f32_f32 = |name_nullterm: &[u8]| -> LLVMValueRef {
            unsafe {
                let float_type = LLVMFloatTypeInContext(context);
                let mut arg_types = [float_type, float_type];
                let fn_type = LLVMFunctionType(float_type, arg_types.as_mut_ptr(), 1, 0);
                LLVMAddFunction(module, name_nullterm.as_ptr() as *const _, fn_type)
            }
        };
        let make_i32_i32_i32 = |name_nullterm: &[u8]| -> LLVMValueRef {
            unsafe {
                let int_type = LLVMInt32TypeInContext(context);
                let mut arg_types = [int_type, int_type];
                let fn_type = LLVMFunctionType(int_type, arg_types.as_mut_ptr(), 1, 0);
                LLVMAddFunction(module, name_nullterm.as_ptr() as *const _, fn_type)
            }
        };

        Self {
            sqrt_f32: make(b"llvm.sqrt.f32\0"),
            sin_f32: make(b"llvm.sin.f32\0"),
            cos_f32: make(b"llvm.cos.f32\0"),
            pow_f32: make_f32_f32_f32(b"llvm.pow.f32\0"),
            powi_i32: make_i32_i32_i32(b"llvm.powi.i32\0"),
            exp_f32: make(b"llvm.exp.f32\0"),
            exp2_f32: make(b"llvm.exp2.f32\0"),
            log_f32: make(b"llvm.log.f32\0"),
            log10_f32: make(b"llvm.log10.f32\0"),
            log2_f32: make(b"llvm.log2.f32\0"),
            fabs_f32: make(b"llvm.fabs.f32\0"),
            floor_f32: make(b"llvm.floor.f32\0"),
            ceil_f32: make(b"llvm.ceil.f32\0"),
            trunc_f32: make(b"llvm.trunc.f32\0"),
        }
    }
}

struct Converter<'a> {
    source: &'a i::Program,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    intrinsics: Intrinsics,
    main_fn: LLVMValueRef,

    value_pointers: HashMap<i::VariableId, LLVMValueRef>,
    input_pointer: LLVMValueRef,
    output_pointer: LLVMValueRef,
    label_blocks: Vec<LLVMBasicBlockRef>,
    current_block_terminated: bool,
}

impl<'a> Converter<'a> {
    fn u32_const(&self, value: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt32TypeInContext(self.context), value as u64, 0) }
    }

    fn i32_const(&self, value: i32) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt32TypeInContext(self.context), value as u64, 1) }
    }

    fn f32_const(&self, value: f32) -> LLVMValueRef {
        unsafe { LLVMConstReal(LLVMFloatTypeInContext(self.context), value as f64) }
    }

    fn b1_const(&self, value: bool) -> LLVMValueRef {
        unsafe {
            LLVMConstInt(
                LLVMInt1TypeInContext(self.context),
                if value { 1 } else { 0 },
                0,
            )
        }
    }

    fn apply_proxy_to_const_indexes(proxy: &[(usize, ProxyMode)], indexes: &[u32]) -> Vec<u32> {
        debug_assert!(proxy.len() == indexes.len());
        let mut result = Vec::new();
        for position in 0..proxy.len() {
            match proxy[position].1 {
                ProxyMode::Keep => result.push(indexes[position]),
                ProxyMode::Collapse => result.push(0),
                ProxyMode::Discard => (),
            }
        }
        result
    }

    fn apply_proxy_to_dyn_indexes(
        &self,
        proxy: &[(usize, ProxyMode)],
        indexes: &[LLVMValueRef],
    ) -> Vec<LLVMValueRef> {
        if indexes.len() == 0 {
            assert!(proxy.len() == 0);
            return Vec::new();
        }
        debug_assert!(proxy.len() + 1 == indexes.len());
        let mut result = Vec::new();
        result.push(indexes[0]);
        for position in 0..proxy.len() {
            match proxy[position].1 {
                ProxyMode::Keep => result.push(indexes[position + 1]),
                ProxyMode::Collapse => result.push(self.u32_const(0)),
                ProxyMode::Discard => (),
            }
        }
        result
    }

    fn store_value(&mut self, value: &i::Value, content: LLVMValueRef, const_indexes: &[u32]) {
        let mut indexes = Vec::new();
        if const_indexes.len() > 0 {
            indexes.push(self.u32_const(0));
            for index in const_indexes {
                indexes.push(self.u32_const(*index));
            }
        }
        self.store_value_dyn(value, content, &mut indexes[..]);
    }

    fn store_value_dyn(
        &mut self,
        value: &i::Value,
        content: LLVMValueRef,
        indexes: &mut [LLVMValueRef],
    ) {
        let mut indexes = self.apply_proxy_to_dyn_indexes(&value.dimensions, indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if indexes.len() > 0 {
                    ptr = unsafe {
                        LLVMBuildGEP(
                            self.builder,
                            ptr,
                            indexes.as_mut_ptr(),
                            indexes.len() as u32,
                            UNNAMED,
                        )
                    };
                } else {
                    assert!(value.dimensions.len() == 0);
                }
                unsafe {
                    LLVMBuildStore(self.builder, content, ptr);
                }
            }
            i::ValueBase::Literal(..) => panic!("Cannot store to a constant."),
        }
    }

    fn load_value_dyn(&mut self, value: &i::Value, indexes: &mut [LLVMValueRef]) -> LLVMValueRef {
        let mut indexes = self.apply_proxy_to_dyn_indexes(&value.dimensions, indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if indexes.len() > 0 {
                    ptr = unsafe {
                        LLVMBuildGEP(
                            self.builder,
                            ptr,
                            indexes.as_mut_ptr(),
                            indexes.len() as u32,
                            UNNAMED,
                        )
                    };
                }
                unsafe { LLVMBuildLoad(self.builder, ptr, UNNAMED) }
            }
            i::ValueBase::Literal(data) => {
                // Last we left it, we were trying to figure out how to index stuff or something
                // like basically figure out what to do to return the requested value of the
                // known data.
                if let i::KnownData::Array(..) = &data {
                    let runtime_value = self.create_temp_value_holding_data(&data);
                    let required_dims = value.get_type(&self.source).collect_dimensions();
                    // +1 for pointer dereference.
                    assert!(required_dims.len() + 1 == indexes.len());
                    unsafe {
                        let value_ptr = LLVMBuildGEP(
                            self.builder,
                            runtime_value,
                            indexes.as_mut_ptr(),
                            indexes.len() as u32,
                            UNNAMED,
                        );
                        LLVMBuildLoad(self.builder, value_ptr, UNNAMED)
                    }
                } else {
                    assert!(indexes.len() == 0, "Cannot index scalar data.");
                    match data {
                        i::KnownData::Array(..) => unreachable!("Handled above."),
                        i::KnownData::Bool(value) => self.b1_const(*value),
                        i::KnownData::Int(value) => self.i32_const(*value as i32),
                        i::KnownData::Float(value) => self.f32_const(*value as f32),
                    }
                }
            }
        }
    }

    fn load_value(&mut self, value: &i::Value, const_indexes: &[u32]) -> LLVMValueRef {
        let const_indexes = Self::apply_proxy_to_const_indexes(&value.dimensions, const_indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if const_indexes.len() > 0 {
                    let mut indices = Vec::new();
                    indices.push(self.u32_const(0));
                    for (index, (_, proxy_mode)) in value.dimensions.iter().enumerate() {
                        match proxy_mode {
                            ProxyMode::Keep => indices.push(self.u32_const(const_indexes[index])),
                            ProxyMode::Collapse => indices.push(self.u32_const(0)),
                            ProxyMode::Discard => (),
                        }
                    }
                    ptr = unsafe {
                        LLVMBuildGEP(
                            self.builder,
                            ptr,
                            indices.as_mut_ptr(),
                            indices.len() as u32,
                            UNNAMED,
                        )
                    };
                }
                unsafe { LLVMBuildLoad(self.builder, ptr, UNNAMED) }
            }
            i::ValueBase::Literal(data) => {
                let mut data = data.clone();
                for index in const_indexes {
                    if let i::KnownData::Array(mut values) = data {
                        data = values.remove(index as usize);
                    } else {
                        unreachable!("Illegal indexes should be caught earlier.");
                    }
                }
                match data {
                    i::KnownData::Array(..) => unimplemented!(),
                    i::KnownData::Bool(value) => self.b1_const(value),
                    i::KnownData::Int(value) => self.i32_const(value as i32),
                    i::KnownData::Float(value) => self.f32_const(value as f32),
                }
            }
        }
    }

    fn store_data_in_ptr(&self, ptr: LLVMValueRef, data: &i::KnownData, current_indexes: &[usize]) {
        if let i::KnownData::Array(items) = data {
            debug_assert!(items.len() > 0);
            let mut new_indexes = Vec::with_capacity(current_indexes.len() + 1);
            for ci in current_indexes {
                new_indexes.push(*ci);
            }
            new_indexes.push(0);
            for item in items {
                self.store_data_in_ptr(ptr, item, &new_indexes[..]);
                let last = new_indexes.len() - 1;
                new_indexes[last] += 1;
            }
        } else {
            let mut ptr = ptr;
            if current_indexes.len() > 0 {
                let mut literal_indexes: Vec<_> = current_indexes
                    .iter()
                    .map(|i| self.u32_const(*i as u32))
                    .collect();
                ptr = unsafe {
                    LLVMBuildGEP(
                        self.builder,
                        ptr,
                        literal_indexes.as_mut_ptr(),
                        literal_indexes.len() as u32,
                        UNNAMED,
                    )
                };
            }
            let value = match data {
                i::KnownData::Bool(value) => self.b1_const(*value),
                i::KnownData::Int(value) => self.i32_const(*value as i32),
                i::KnownData::Float(value) => self.f32_const(*value as f32),
                i::KnownData::Array(..) => unreachable!("Handled above."),
            };
            unsafe {
                LLVMBuildStore(self.builder, value, ptr);
            }
        }
    }

    fn create_temp_value_holding_data(&self, data: &i::KnownData) -> LLVMValueRef {
        let dtype = data.get_type();
        let vtype = llvm_type(self.context, &dtype);
        let value_ptr = unsafe { LLVMBuildAlloca(self.builder, vtype, UNNAMED) };
        self.store_data_in_ptr(value_ptr, data, &[]);
        value_ptr
    }

    fn get_block_for_label(&self, id: &i::LabelId) -> LLVMBasicBlockRef {
        self.label_blocks[id.raw()]
    }

    fn usize_vec_to_u32(vec: Vec<usize>) -> Vec<u32> {
        vec.into_iter().map(|i| i as u32).collect()
    }

    fn convert_unary_expression(&mut self, op: &i::UnaryOperator, a: &i::Value, x: &i::Value) {
        let dimensions = x.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let ar = self.load_value(a, &coord[..]);

            let xr = self.do_unary_op(op, ar);
            self.store_value(x, xr, &coord[..]);
        }
    }

    fn build_call(&mut self, fn_ref: LLVMValueRef, args: &mut [LLVMValueRef]) -> LLVMValueRef {
        unsafe {
            LLVMBuildCall(
                self.builder,
                fn_ref,
                args.as_mut_ptr(),
                args.len() as u32,
                UNNAMED,
            )
        }
    }

    fn do_unary_op(&mut self, op: &i::UnaryOperator, ar: LLVMValueRef) -> LLVMValueRef {
        match op {
            i::UnaryOperator::BNot => unsafe {
                LLVMBuildXor(self.builder, ar, self.u32_const(0xFFFFFFFF), UNNAMED)
            },
            i::UnaryOperator::FAbs => self.build_call(self.intrinsics.fabs_f32, &mut [ar]),
            i::UnaryOperator::FCeil => self.build_call(self.intrinsics.ceil_f32, &mut [ar]),
            i::UnaryOperator::FCos => self.build_call(self.intrinsics.cos_f32, &mut [ar]),
            i::UnaryOperator::FExp => self.build_call(self.intrinsics.exp_f32, &mut [ar]),
            i::UnaryOperator::FExp2 => self.build_call(self.intrinsics.exp2_f32, &mut [ar]),
            i::UnaryOperator::FFloor => self.build_call(self.intrinsics.floor_f32, &mut [ar]),
            i::UnaryOperator::FLog => self.build_call(self.intrinsics.log_f32, &mut [ar]),
            i::UnaryOperator::FLog10 => self.build_call(self.intrinsics.log10_f32, &mut [ar]),
            i::UnaryOperator::FLog2 => self.build_call(self.intrinsics.log2_f32, &mut [ar]),
            i::UnaryOperator::FSin => self.build_call(self.intrinsics.sin_f32, &mut [ar]),
            i::UnaryOperator::FSqrt => self.build_call(self.intrinsics.sqrt_f32, &mut [ar]),
            i::UnaryOperator::FTrunc => self.build_call(self.intrinsics.trunc_f32, &mut [ar]),
            i::UnaryOperator::IAbs => unimplemented!(),
            i::UnaryOperator::NegF => unsafe {
                LLVMBuildFSub(self.builder, self.f32_const(0.0), ar, UNNAMED)
            },
            i::UnaryOperator::NegI => unsafe {
                LLVMBuildSub(self.builder, self.i32_const(0), ar, UNNAMED)
            },
            i::UnaryOperator::Not => unsafe {
                LLVMBuildXor(self.builder, ar, self.b1_const(true), UNNAMED)
            },
        }
    }

    fn convert_binary_expression(
        &mut self,
        op: &i::BinaryOperator,
        a: &i::Value,
        b: &i::Value,
        x: &i::Value,
    ) {
        let dimensions = x.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let ar = self.load_value(a, &coord[..]);
            let br = self.load_value(b, &coord[..]);

            let xr = self.do_binary_op(op, ar, br);
            self.store_value(x, xr, &coord[..]);
        }
    }

    fn do_binary_op(
        &mut self,
        op: &i::BinaryOperator,
        ar: LLVMValueRef,
        br: LLVMValueRef,
    ) -> LLVMValueRef {
        match op {
            i::BinaryOperator::AddI => unsafe { LLVMBuildAdd(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::SubI => unsafe { LLVMBuildSub(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::MulI => unsafe { LLVMBuildMul(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::DivI => unsafe { LLVMBuildSDiv(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::ModI => unsafe { LLVMBuildSRem(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::AddF => unsafe { LLVMBuildFAdd(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::SubF => unsafe { LLVMBuildFSub(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::MulF => unsafe { LLVMBuildFMul(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::DivF => unsafe { LLVMBuildFDiv(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::ModF => unsafe { LLVMBuildFRem(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::CompI(condition) => {
                let predicate = match condition {
                    i::Condition::Equal => LLVMIntPredicate::LLVMIntEQ,
                    i::Condition::NotEqual => LLVMIntPredicate::LLVMIntNE,
                    i::Condition::GreaterThan => LLVMIntPredicate::LLVMIntSGT,
                    i::Condition::GreaterThanOrEqual => LLVMIntPredicate::LLVMIntSGE,
                    i::Condition::LessThan => LLVMIntPredicate::LLVMIntSLT,
                    i::Condition::LessThanOrEqual => LLVMIntPredicate::LLVMIntSLE,
                };
                unsafe { LLVMBuildICmp(self.builder, predicate, ar, br, UNNAMED) }
            }
            i::BinaryOperator::CompF(condition) => {
                let predicate = match condition {
                    i::Condition::Equal => LLVMRealPredicate::LLVMRealOEQ,
                    i::Condition::NotEqual => LLVMRealPredicate::LLVMRealONE,
                    i::Condition::GreaterThan => LLVMRealPredicate::LLVMRealOGT,
                    i::Condition::GreaterThanOrEqual => LLVMRealPredicate::LLVMRealOGE,
                    i::Condition::LessThan => LLVMRealPredicate::LLVMRealOLT,
                    i::Condition::LessThanOrEqual => LLVMRealPredicate::LLVMRealOLE,
                };
                unsafe { LLVMBuildFCmp(self.builder, predicate, ar, br, UNNAMED) }
            }
            i::BinaryOperator::BAnd => unsafe { LLVMBuildAnd(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::BOr => unsafe { LLVMBuildOr(self.builder, ar, br, UNNAMED) },
            i::BinaryOperator::BXor => unsafe { LLVMBuildXor(self.builder, ar, br, UNNAMED) },
        }
    }

    fn convert_move(&mut self, from: &i::Value, to: &i::Value) {
        let dimensions = to.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let from = self.load_value(from, &coord[..]);
            self.store_value(to, from, &coord[..]);
        }
    }

    fn convert_store(&mut self, from: &i::Value, to: &i::Value, to_indexes: &Vec<i::Value>) {
        let dimensions = from.dimensions.iter().map(|(len, _)| *len).collect();
        let mut dyn_indexes: Vec<_> = to_indexes
            .iter()
            .map(|value| self.load_value(value, &[]))
            .collect();
        dyn_indexes.insert(0, self.u32_const(0));
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let mut to_indexes = dyn_indexes.clone();
            for static_index in &coord {
                to_indexes.push(self.u32_const(*static_index));
            }
            let item = self.load_value(from, &coord[..]);
            self.store_value_dyn(to, item, &mut to_indexes[..]);
        }
    }

    fn convert_load(&mut self, from: &i::Value, from_indexes: &Vec<i::Value>, to: &i::Value) {
        let dimensions = to.dimensions.iter().map(|(len, _)| *len).collect();
        let mut dyn_indexes: Vec<_> = from_indexes
            .iter()
            .map(|value| self.load_value(value, &[]))
            .collect();
        dyn_indexes.insert(0, self.u32_const(0));
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let mut from_indexes = dyn_indexes.clone();
            for static_index in &coord {
                from_indexes.push(self.u32_const(*static_index));
            }
            let item = self.load_value_dyn(from, &mut from_indexes[..]);
            self.store_value(to, item, &coord[..]);
        }
    }

    fn convert_label(&mut self, id: &i::LabelId) {
        unsafe {
            if !self.current_block_terminated {
                LLVMBuildBr(self.builder, self.get_block_for_label(id));
            }
            LLVMPositionBuilderAtEnd(self.builder, self.get_block_for_label(id));
        }
        self.current_block_terminated = false;
    }

    fn convert_branch(
        &mut self,
        condition: &i::Value,
        true_target: &i::LabelId,
        false_target: &i::LabelId,
    ) {
        unsafe {
            LLVMBuildCondBr(
                self.builder,
                self.load_value(condition, &[]),
                self.get_block_for_label(true_target),
                self.get_block_for_label(false_target),
            );
        }
        self.current_block_terminated = true;
    }

    fn convert_abort(&mut self, error_code: u32) {
        unsafe {
            LLVMBuildRet(self.builder, self.u32_const(error_code));
        }
        self.current_block_terminated = true;
    }

    fn convert_jump(&mut self, label: &i::LabelId) {
        unsafe {
            LLVMBuildBr(self.builder, self.get_block_for_label(label));
        }
        self.current_block_terminated = true;
    }

    fn create_param_pointers(&mut self) {
        for (index, input) in self.source.borrow_inputs().iter().enumerate() {
            let mut indices = [self.u32_const(0), self.u32_const(index as u32)];
            let input_ptr = unsafe {
                LLVMBuildGEP(
                    self.builder,
                    self.input_pointer,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    UNNAMED,
                )
            };
            self.value_pointers.insert(*input, input_ptr);
        }
        for (index, output) in self.source.borrow_outputs().iter().enumerate() {
            let mut indices = [self.u32_const(0), self.u32_const(index as u32)];
            let output_ptr = unsafe {
                LLVMBuildGEP(
                    self.builder,
                    self.output_pointer,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    UNNAMED,
                )
            };
            self.value_pointers.insert(*output, output_ptr);
        }
    }

    fn create_variable_pointers(&mut self) {
        let (ins, outs) = (self.source.borrow_inputs(), self.source.borrow_outputs());
        let param_set: HashSet<_> = ins.iter().chain(outs.iter()).collect();
        for var_id in self.source.iterate_all_variables() {
            if param_set.contains(&var_id) {
                continue;
            }
            let llvmt = llvm_type(self.context, self.source[var_id].borrow_type());
            let var_ptr = unsafe { LLVMBuildAlloca(self.builder, llvmt, UNNAMED) };
            self.value_pointers.insert(var_id, var_ptr);
        }
    }

    fn create_blocks_for_labels(&mut self) {
        for label_index in 0..self.source.get_num_labels() {
            self.label_blocks.push(unsafe {
                LLVMAppendBasicBlockInContext(
                    self.context,
                    self.main_fn,
                    format!("l{}\0", label_index).as_ptr() as *const _,
                )
            });
        }
    }

    fn optimize(&mut self) {
        unsafe {
            debug_assert!(
                llvm_sys::analysis::LLVMVerifyModule(
                    self.module,
                    llvm_sys::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
                    std::ptr::null_mut()
                ) == 0,
                "Module failed to verify."
            );

            let pm = LLVMCreatePassManager();
            // Convert all our stores / loads into flat, efficient SSA style code.
            llvmt::scalar::LLVMAddScalarReplAggregatesPassSSA(pm);
            llvmt::scalar::LLVMAddEarlyCSEPass(pm);
            llvmt::scalar::LLVMAddInstructionCombiningPass(pm);
            llvmt::scalar::LLVMAddReassociatePass(pm);
            llvmt::scalar::LLVMAddGVNPass(pm);
            llvmt::scalar::LLVMAddCFGSimplificationPass(pm);

            LLVMRunPassManager(pm, self.module);
            LLVMDisposePassManager(pm);
        }
    }

    fn convert(&mut self) {
        self.create_param_pointers();
        self.create_variable_pointers();
        self.create_blocks_for_labels();

        for instruction in self.source.borrow_instructions().clone() {
            match instruction {
                i::Instruction::Abort(error_code) => self.convert_abort(*error_code),
                i::Instruction::BinaryOperation { op, a, b, x } => {
                    self.convert_binary_expression(op, a, b, x)
                }
                i::Instruction::UnaryOperation { op, a, x } => {
                    self.convert_unary_expression(op, a, x)
                }
                i::Instruction::Move { from, to } => self.convert_move(from, to),
                i::Instruction::Label(id) => self.convert_label(id),
                i::Instruction::Branch {
                    condition,
                    true_target,
                    false_target,
                } => self.convert_branch(condition, true_target, false_target),
                i::Instruction::Jump { label } => self.convert_jump(label),
                i::Instruction::Store {
                    from,
                    to,
                    to_indexes,
                } => self.convert_store(from, to, to_indexes),
                i::Instruction::Load {
                    from,
                    from_indexes,
                    to,
                } => self.convert_load(from, from_indexes, to),
            }
        }

        unsafe {
            LLVMBuildRet(self.builder, self.u32_const(0));
            LLVMDisposeBuilder(self.builder);

            #[cfg(feature = "dump-llvmir")]
            {
                // Dump human-readable IR to stdout
                println!("\nUnoptimized:");
                LLVMDumpModule(self.module);
            }
        }

        println!("Starting optimization.");
        self.optimize();

        #[cfg(feature = "dump-llvmir")]
        unsafe {
            println!("\nOptimized:");
            LLVMDumpModule(self.module);
        }
    }
}

fn llvm_type(context: LLVMContextRef, trivial_type: &i::DataType) -> LLVMTypeRef {
    unsafe {
        match trivial_type {
            i::DataType::B1 => LLVMInt1TypeInContext(context),
            i::DataType::I32 => LLVMInt32TypeInContext(context),
            i::DataType::F32 => LLVMFloatTypeInContext(context),
            i::DataType::Array(len, etype) => LLVMArrayType(llvm_type(context, etype), *len as u32),
        }
    }
}

pub fn ingest(source: &i::Program) -> o::Program {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(b"nsprog\0".as_ptr() as *const _, context);
        let builder = LLVMCreateBuilderInContext(context);

        let mut input_types = Vec::new();
        for input in source.borrow_inputs() {
            let var = source.borrow_variable(*input);
            input_types.push(llvm_type(context, var.borrow_type()));
        }
        let input_data_type = LLVMStructTypeInContext(
            context,
            input_types.as_mut_ptr(),
            input_types.len() as u32,
            0,
        );
        let input_pointer_type = LLVMPointerType(input_data_type, 0);

        let mut output_types = Vec::new();
        for output in source.borrow_outputs() {
            let var = source.borrow_variable(*output);
            output_types.push(llvm_type(context, var.borrow_type()));
        }
        let output_data_type = LLVMStructTypeInContext(
            context,
            output_types.as_mut_ptr(),
            output_types.len() as u32,
            0,
        );
        let output_pointer_type = LLVMPointerType(output_data_type, 0);

        let i32t = LLVMInt32TypeInContext(context);
        let mut argts = [input_pointer_type, output_pointer_type];
        let function_type = LLVMFunctionType(i32t, argts.as_mut_ptr(), argts.len() as u32, 0);
        let main_fn = LLVMAddFunction(module, b"main\0".as_ptr() as *const _, function_type);
        let entry_block =
            LLVMAppendBasicBlockInContext(context, main_fn, b"entry\0".as_ptr() as *const _);
        LLVMPositionBuilderAtEnd(builder, entry_block);
        let input_pointer = LLVMGetParam(main_fn, 0);
        let output_pointer = LLVMGetParam(main_fn, 1);

        let intrinsics = Intrinsics::new(module, context);

        let mut converter = Converter {
            source,
            context,
            module,
            builder,
            intrinsics,
            main_fn,

            value_pointers: HashMap::new(),
            input_pointer,
            output_pointer,
            label_blocks: Vec::with_capacity(source.get_num_labels()),
            current_block_terminated: false,
        };

        converter.convert();

        let Converter {
            context, module, ..
        } = converter;
        o::Program::new(
            context,
            module,
            input_data_type,
            output_data_type,
            source.borrow_error_descriptions().clone(),
        )
    }
}
