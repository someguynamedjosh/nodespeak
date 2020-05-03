use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::ptr;

pub struct Program {
    execution_engine: LLVMExecutionEngineRef,
    function: extern "C" fn(*mut u8, *mut u8),
    context: LLVMContextRef,
    module: LLVMModuleRef,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        unsafe {
            let content = LLVMPrintModuleToString(self.module);
            write!(formatter, "{}", std::ffi::CStr::from_ptr(content).to_string_lossy())?;
            LLVMDisposeMessage(content);
        }
        write!(formatter, "")
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            // Disposing the execution engine also disposes the module.
            LLVMDisposeExecutionEngine(self.execution_engine);
            LLVMContextDispose(self.context);
        }
    }
}

impl Program {
    /// After this, the prrogram will handle dropping the module and context automatically.
    pub fn new(context: LLVMContextRef, module: LLVMModuleRef) -> Self {
        let execution_engine = unsafe {
            let mut ee_ref = MaybeUninit::uninit();
            let mut creation_error = ptr::null_mut();
            LLVMLinkInMCJIT();
            assert!(
                LLVM_InitializeNativeTarget() != 1,
                "Failed to initialize native target."
            );
            assert!(
                LLVM_InitializeNativeAsmPrinter() != 1,
                "Failed to initialize native asm."
            );
            // This takes ownership of the module so disposing the EE disposes the module.
            LLVMCreateExecutionEngineForModule(ee_ref.as_mut_ptr(), module, &mut creation_error);

            ee_ref.assume_init()
        };
        let function = unsafe {
            let func_addr =
                LLVMGetFunctionAddress(execution_engine, b"main\0".as_ptr() as *const _);
            mem::transmute(func_addr)
        };
        Self {
            execution_engine,
            function,
            context,
            module,
        }
    }

    pub fn execute(&self, input_data: &mut [u8], output_data: &mut [u8]) {
        (self.function)(input_data.as_mut_ptr(), output_data.as_mut_ptr());
    }
}
