use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::ops::{Index, IndexMut};

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
    pub(super) fn new(code_size: usize, storage_size: usize) -> Program {
        Program {
            code: CodeBlock::new(code_size),
            storage: StorageBlock::new(storage_size),
        }
    }

    pub(super) unsafe fn execute(&self) -> i64 {
        self.code.execute()
    }

    pub(super) fn write_u8_to_code(&mut self, index: usize, value: u8) {
        self.code[index] = value;
    }

    pub(super) fn write_iter_to_code<'a>(
        &mut self,
        mut index: usize,
        values: impl Iterator<Item = &'a u8>,
    ) {
        for value in values {
            self.code[index] = *value;
            index += 1;
        }
    }

    pub(super) fn write_u64_to_code(&mut self, index: usize, value: u64) {
        self.write_iter_to_code(index, value.to_le_bytes().into_iter());
    }

    pub(super) fn write_u8_to_storage(&mut self, index: usize, value: u8) {
        self.storage[index] = value;
    }

    pub(super) fn write_iter_to_storage<'a>(
        &mut self,
        mut index: usize,
        values: impl Iterator<Item = &'a u8>,
    ) {
        for value in values {
            self.storage[index] = *value;
            index += 1;
        }
    }

    pub(super) fn write_u64_to_storage(&mut self, index: usize, value: u64) {
        self.write_iter_to_storage(index, value.to_le_bytes().into_iter());
    }

    pub(super) fn get_storage_address(&mut self, index: usize) -> usize {
        self.storage.get_address() + index
    }
}

mod test {
    #[test]
    fn simple_return_function() {
        use super::Program;
        let mut program = Program::new(11, 8);

        // Function that returns the first 8 bytes of the storage block.
        program.write_iter_to_code(
            0,
            [
                0x48, 0xA1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // mov
                0xC3, // rtn
            ]
            .into_iter(),
        );
        // Set the address for the mov instruction to the start of the storage block.
        let address = program.get_storage_address(0) as u64;
        program.write_u64_to_code(2, address);
        // Put some data in the storage block.
        program.write_u64_to_storage(0, 0x0011223344556677);

        unsafe {
            assert!(program.execute() == 0x0011223344556677);
        }
    }
}
