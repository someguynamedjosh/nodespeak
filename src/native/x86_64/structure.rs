use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::ops::{Index, IndexMut};

use crate::native::traits;
pub use crate::shared::{NativeBaseType, NativeType};
use crate::specialized::structure as i;

const PAGE_SIZE: usize = 4096;

struct CodeBlock {
    contents: *mut u8,
    size: usize,
}

impl Index<usize> for CodeBlock {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        assert!(index < self.size);
        unsafe { &*self.contents.offset(index as isize) }
    }
}

impl IndexMut<usize> for CodeBlock {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        assert!(index < self.size);
        unsafe { &mut *self.contents.offset(index as isize) }
    }
}

impl Drop for CodeBlock {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.contents as *mut libc::c_void);
        }
    }
}

impl CodeBlock {
    fn new(size: usize) -> CodeBlock {
        let num_pages = (size / PAGE_SIZE + 1) * PAGE_SIZE;
        unsafe {
            let num_bytes = num_pages * PAGE_SIZE;
            let mut memory_block: *mut libc::c_void = mem::uninitialized();
            libc::posix_memalign(&mut memory_block, PAGE_SIZE, num_bytes);
            libc::mprotect(
                memory_block,
                num_bytes,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
            );
            // Fill with 0xc3 RET command to help soften the blow of trivial errors.
            libc::memset(memory_block, 0xc3, num_bytes);
            CodeBlock {
                contents: mem::transmute(memory_block),
                size,
            }
        }
    }

    unsafe fn execute(&self) -> i64 {
        let function_pointer: (fn() -> i64) = mem::transmute(self.contents);
        function_pointer()
    }
}

struct StorageBlock {
    contents: *mut u8,
    size: usize,
}

impl Index<usize> for StorageBlock {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        assert!(index < self.size);
        unsafe { &*self.contents.offset(index as isize) }
    }
}

impl IndexMut<usize> for StorageBlock {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        assert!(index < self.size);
        unsafe { &mut *self.contents.offset(index as isize) }
    }
}

impl Drop for StorageBlock {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.contents as *mut libc::c_void);
        }
    }
}

impl StorageBlock {
    fn new(size: usize) -> StorageBlock {
        let num_pages = (size / PAGE_SIZE + 1) * PAGE_SIZE;
        unsafe {
            let num_bytes = num_pages * PAGE_SIZE;
            let mut memory_block: *mut libc::c_void = mem::uninitialized();
            libc::posix_memalign(&mut memory_block, PAGE_SIZE, num_bytes);
            libc::mprotect(memory_block, num_bytes, libc::PROT_READ | libc::PROT_WRITE);
            // Fill with 0xc3 RET command to help soften the blow of trivial errors.
            libc::memset(memory_block, 0xc3, num_bytes);
            StorageBlock {
                contents: mem::transmute(memory_block),
                size,
            }
        }
    }

    fn get_address(&self) -> usize {
        self.contents as usize
    }
}

pub struct Program {
    code: CodeBlock,
    storage: StorageBlock,
    // Data type and address.
    inputs: Vec<(NativeType, usize)>,
    outputs: Vec<(NativeType, usize)>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "{} inputs: {:?}", self.inputs.len(), self.inputs)?;
        writeln!(
            formatter,
            "{} outputs: {:?}",
            self.outputs.len(),
            self.outputs
        )?;
        write!(formatter, "{} code bytes:", self.code.size)?;
        for byte_index in 0..self.code.size {
            if byte_index % 8 == 0 {
                write!(formatter, "\n{:02X}", self.code[byte_index])?;
            } else {
                write!(formatter, " {:02X}", self.code[byte_index])?;
            }
        }

        writeln!(formatter, "")?;
        write!(formatter, "{} storage bytes:", self.storage.size)?;
        for byte_index in 0..self.storage.size {
            if byte_index % 8 == 0 {
                write!(formatter, "\n{:02X}", self.storage[byte_index])?;
            } else {
                write!(formatter, " {:02X}", self.storage[byte_index])?;
            }
        }

        write!(formatter, "")
    }
}

impl Program {
    pub(super) fn new(
        code_size: usize,
        storage_size: usize,
        inputs: Vec<(NativeType, usize)>,
        outputs: Vec<(NativeType, usize)>,
    ) -> Program {
        // Ensure that there is enough space to fit all the inputs and outputs.
        for input in &inputs {
            debug_assert!(
                storage_size - input.1 >= input.0.get_physical_size() as usize,
                concat!(
                    "There is not enough space to hold an input. (Address {}, length {}, ",
                    "storage size {}.)"
                ),
                input.1,
                input.0.get_physical_size(),
                storage_size
            );
        }
        for output in &outputs {
            debug_assert!(
                storage_size - output.1 >= output.0.get_physical_size() as usize,
                concat!(
                    "There is not enough space to hold an output. (Address {}, length {}, ",
                    "storage size {}.)"
                ),
                output.1,
                output.0.get_physical_size(),
                storage_size
            );
        }
        Program {
            code: CodeBlock::new(code_size),
            storage: StorageBlock::new(storage_size),
            inputs,
            outputs,
        }
    }

    pub(super) unsafe fn execute(&self) -> i64 {
        self.code.execute()
    }

    pub(super) unsafe fn write_iter_to_code<'a>(
        &mut self,
        mut index: usize,
        values: impl Iterator<Item = &'a u8>,
    ) {
        for value in values {
            self.code[index] = *value;
            index += 1;
        }
    }

    pub(super) fn get_storage_address(&self) -> usize {
        self.storage.get_address()
    }

    unsafe fn write_iter_to_storage<'a>(
        &mut self,
        mut index: usize,
        values: impl Iterator<Item = &'a u8>,
    ) {
        for value in values {
            self.storage[index] = *value;
            index += 1;
        }
    }
}

impl traits::Program for Program {
    fn new(input: &i::Program) -> Program {
        super::ingest::ingest(input)
    }

    unsafe fn execute(&self) -> i64 {
        Program::execute(self)
    }

    fn set_input_i32(&mut self, index: usize, value: i32) {
        assert!(index < self.inputs.len());
        assert!(self.inputs[index].0 == NativeType::int_scalar());
        let address = self.inputs[index].1;
        // Safe based on the assumption that the creator of the program allocated enough space
        // to hold the variable.
        unsafe {
            self.write_iter_to_storage(address, value.to_le_bytes().iter());
        }
    }

    fn set_input_f32(&mut self, index: usize, value: f32) {
        assert!(index < self.inputs.len());
        assert!(self.inputs[index].0 == NativeType::float_scalar());
        let address = self.inputs[index].1;
        // Safe based on the assumption that the creator of the program allocated enough space
        // to hold the variable.
        unsafe {
            self.write_iter_to_storage(address, value.to_le_bytes().iter());
        }
    }

    fn read_output_i32(&self, index: usize) -> i32 {
        assert!(index < self.outputs.len());
        assert!(self.outputs[index].0 == NativeType::int_scalar());
        let address = self.outputs[index].1;
        let bytes = [
            self.storage[address + 0],
            self.storage[address + 1],
            self.storage[address + 2],
            self.storage[address + 3],
        ];
        i32::from_le_bytes(bytes)
    }

    fn read_output_f32(&self, index: usize) -> f32 {
        assert!(index < self.outputs.len());
        assert!(self.outputs[index].0 == NativeType::float_scalar());
        let address = self.outputs[index].1;
        let bytes = [
            self.storage[address + 0],
            self.storage[address + 1],
            self.storage[address + 2],
            self.storage[address + 3],
        ];
        f32::from_le_bytes(bytes)
    }

    fn list_inputs(&self) -> Vec<NativeType> {
        self.inputs.iter().map(|var| var.0.clone()).collect()
    }

    fn list_outputs(&self) -> Vec<NativeType> {
        self.outputs.iter().map(|var| var.0.clone()).collect()
    }
}
