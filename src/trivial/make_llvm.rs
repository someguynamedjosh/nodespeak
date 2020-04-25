use super::structure as i;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::collections::HashMap;

const UNNAMED: *const libc::c_char = b"\0".as_ptr() as *const libc::c_char;

struct Converter<'a> {
    source: &'a i::Program,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,

    value_map: HashMap<i::VariableId, DataLocation>,
    input_pointer: LLVMValueRef,
    i32t: LLVMTypeRef,
}

enum DataLocation {
    ValueRef(LLVMValueRef),
    Input { index: u32 },
}

impl<'a> Converter<'a> {
    fn store_value(&mut self, of: &i::Value, vref: LLVMValueRef) {
        if let i::ValueBase::Variable(id) = &of.base {
            // TODO: Check for existing content.
            self.value_map.insert(*id, DataLocation::ValueRef(vref));
        } else {
            unimplemented!()
        }
    }

    fn u32_const(&self, value: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i32t, value as u64, 0) }
    }

    fn get_llvm_value_ref(&mut self, of: &i::Value) -> LLVMValueRef {
        if let i::ValueBase::Variable(id) = &of.base {
            match self.value_map.get(id).unwrap() {
                DataLocation::ValueRef(vref) => vref.clone(),
                DataLocation::Input { index } => {
                    let mut indices = [self.u32_const(0), self.u32_const(*index)];
                    let input_ptr = unsafe {
                        LLVMBuildGEP(
                            self.builder,
                            self.input_pointer,
                            indices.as_mut_ptr(),
                            indices.len() as u32,
                            UNNAMED,
                        )
                    };
                    unsafe {
                        LLVMBuildLoad(
                            self.builder,
                            input_ptr,
                            UNNAMED,
                        )
                    }
                }
            }
        } else {
            unimplemented!()
        }
    }

    fn convert_binary_expression(
        &mut self,
        op: &i::BinaryOperator,
        a: &i::Value,
        b: &i::Value,
        x: &i::Value,
    ) {
        let result = match op {
            i::BinaryOperator::AddI => {
                let ar = self.get_llvm_value_ref(a);
                let br = self.get_llvm_value_ref(b);
                unsafe {
                    LLVMBuildAdd(self.builder, ar, br, UNNAMED)
                }
            }
            _ => unimplemented!(),
        };
        self.store_value(x, result);
    }

    fn convert_move(&mut self, from: &i::Value, to: &i::Value) {
        let from = self.get_llvm_value_ref(from);
        self.store_value(to, from);
    }

    fn convert(mut self) {
        for instruction in self.source.borrow_instructions().clone() {
            match instruction {
                i::Instruction::BinaryOperation { op, a, b, x } => {
                    self.convert_binary_expression(op, a, b, x);
                }
                i::Instruction::Move { from, to } => {
                    self.convert_move(from, to);
                }
                _ => unimplemented!(),
            }
        }

        unsafe {
            LLVMBuildRetVoid(self.builder);
            LLVMDisposeBuilder(self.builder);

            // Dump human-readable IR to stdout
            LLVMDumpModule(self.module);
        }

        unsafe {
            LLVMContextDispose(self.context);
        }
    }
}

pub fn make_llvm(source: &i::Program) {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(b"nsprog\0".as_ptr() as *const _, context);
        let builder = LLVMCreateBuilderInContext(context);

        let i1t = LLVMInt1TypeInContext(context);
        let i32t = LLVMInt32TypeInContext(context);
        let f32t = LLVMFloatTypeInContext(context);
        let mut value_map = HashMap::new();
        let mut input_types = Vec::new();
        for (index, input) in source.borrow_inputs().iter().enumerate() {
            let var = source.borrow_variable(*input);
            let vtype = var.borrow_type();
            assert!(var.borrow_dimensions().len() == 0);
            input_types.push(match vtype.get_base() {
                i::BaseType::B1 => i1t,
                i::BaseType::I32 => i32t,
                i::BaseType::F32 => f32t,
            });
            value_map.insert(
                *input,
                DataLocation::Input {
                    index: index as u32,
                },
            );
        }
        let input_data_type = LLVMStructType(input_types.as_mut_ptr(), input_types.len() as u32, 0);
        let voidt = LLVMVoidTypeInContext(context);
        let input_pointer_type = LLVMPointerType(input_data_type, 0);
        let mut argts = [input_pointer_type];
        let function_type = LLVMFunctionType(voidt, argts.as_mut_ptr(), argts.len() as u32, 0);
        let main_fn = LLVMAddFunction(module, b"main\0".as_ptr() as *const _, function_type);
        let entry_block =
            LLVMAppendBasicBlockInContext(context, main_fn, b"entry\0".as_ptr() as *const _);
        LLVMPositionBuilderAtEnd(builder, entry_block);
        let input_pointer = LLVMGetParam(main_fn, 0);

        let converter = Converter {
            source: source,
            context,
            module,
            builder,

            value_map,
            input_pointer,
            i32t,
        };

        converter.convert()
    }
}
