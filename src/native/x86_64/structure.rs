use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::ops::{Index, IndexMut};

use crate::native::traits;
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
    input_addresses: Vec<usize>,
    output_addresses: Vec<usize>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
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
        input_addresses: Vec<usize>,
        output_addresses: Vec<usize>,
    ) -> Program {
        Program {
            code: CodeBlock::new(code_size),
            storage: StorageBlock::new(storage_size),
            input_addresses,
            output_addresses,
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

    // TODO: Type checking.
    unsafe fn set_input_i32(&mut self, index: usize, value: i32) {
        assert!(index < self.input_addresses.len());
        let address = self.input_addresses[index];
        self.write_iter_to_storage(address, value.to_le_bytes().iter());
    }

    // TODO: Type checking.
    unsafe fn read_output_i32(&self, index: usize) -> i32 {
        assert!(index < self.output_addresses.len());
        let address = self.output_addresses[index];
        let bytes = [
            self.storage[address + 0],
            self.storage[address + 1],
            self.storage[address + 2],
            self.storage[address + 3],
        ];
        i32::from_le_bytes(bytes)
    }
}
