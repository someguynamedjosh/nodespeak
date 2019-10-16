use std::mem;
use std::ops::{Index, IndexMut};

const PAGE_SIZE: usize = 4096;

pub struct CodeBlock {
    contents: *mut u8,
    size: usize,
}

impl Index<usize> for CodeBlock {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        assert!(index < self.size);
        unsafe {
            &*self.contents.offset(index as isize)
        }
    }
}

impl IndexMut<usize> for CodeBlock {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        assert!(index < self.size);
        unsafe {
            &mut *self.contents.offset(index as isize)
        }
    }
}

impl CodeBlock {
    pub fn new(size: usize) -> CodeBlock {
        let num_pages = (size / PAGE_SIZE + 1) * PAGE_SIZE;
        unsafe {
            let num_bytes = num_pages * PAGE_SIZE;
            let mut memory_block: *mut libc::c_void = mem::uninitialized();
            libc::posix_memalign(&mut memory_block, PAGE_SIZE, num_bytes);
            libc::mprotect(memory_block, num_bytes, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
            // Fill with 0xc3 RET command to help soften the blow of trivial errors.
            libc::memset(memory_block, 0xc3, num_bytes);
            CodeBlock {
                contents: mem::transmute(memory_block),
                size
            }
        }
    }

    pub fn get_address(&self) -> usize {
        self.contents as usize
    }

    pub unsafe fn execute(&self) -> i64 {
        let function_pointer: (fn() -> i64) = mem::transmute(self.contents);
        function_pointer()
    }
}

mod test {
    #[test]
    fn simple_return_function() {
        use super::CodeBlock;
        let mut block = CodeBlock::new(11);
        // Function that returns the first 8 bytes of itself.
        let mut machine_code: [u8; 11] = [
            0x48, 0xA1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // mov
            0xC3                                                        // rtn
        ];
        // Set the target address of the mov to be the start of the function.
        let mut address = block.get_address();
        for index in 2..11 {
            machine_code[index] = (address & 0xFF) as u8;
            address = address >> 8;
        }
        for (index, byte) in machine_code.into_iter().enumerate() {
            block[index] = *byte;
        }

        unsafe {
            // The first byte should be a mov instruction.
            assert!(block.execute() & 0xFF == 0x48);
        }
    }
}