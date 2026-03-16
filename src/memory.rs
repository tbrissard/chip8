/// Total addresable memory
const MEMORY_SIZE: usize = 4096;

#[derive(Debug)]
struct Memory {
    mem: [u8; MEMORY_SIZE],
}

impl std::default::Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0u8; MEMORY_SIZE],
        }
    }
}
