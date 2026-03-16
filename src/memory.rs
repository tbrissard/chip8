pub(crate) type Address = u16;
const MEMORY_SIZE: Address = 0xFFF;

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Debug)]
struct Memory {
    mem: [u8; MEMORY_SIZE as usize],
}

impl Memory {
    fn is_reserved(addr: Address) -> bool {
        const RESERVED_SPACE_START: Address = 0x000;
        const RESERVED_SPACE_END: Address = 0x1FF;
        addr >= RESERVED_SPACE_START && addr <= RESERVED_SPACE_END
    }

    /// Write the bytes from bytes at addr
    pub(crate) fn store(&mut self, bytes: &[u8], addr: Address) -> Result<()> {
        if Memory::is_reserved(addr) {
            return Err(MemoryError::WriteError(WriteError::ReservedAddr(addr)));
        }

        if addr as usize + bytes.len() - 1 > MEMORY_SIZE as usize {
            return Err(MemoryError::WriteError(WriteError::OutOfBound(addr)));
        }

        for (b, a) in bytes.iter().zip(addr..) {
            self.mem[a as usize] = *b;
        }

        Ok(())
    }
}

impl std::default::Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0u8; MEMORY_SIZE as usize],
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MemoryError {
    #[error("write error: {0}")]
    WriteError(WriteError),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WriteError {
    #[error("address is out of bound ({0})")]
    OutOfBound(Address),
    #[error("address is reserved ({0})")]
    ReservedAddr(Address),
}

#[cfg(test)]
mod tests {
    use crate::memory::MEMORY_SIZE;
    use crate::memory::MemoryError;

    use super::Memory;
    use super::WriteError;

    fn create_memory() -> Memory {
        Memory::default()
    }

    #[test]
    fn test_write_address_out_of_bounds() {
        let mut mem = create_memory();
        let res = mem.store(&[1u8], MEMORY_SIZE + 1);
        assert_eq!(
            res,
            Err(MemoryError::WriteError(WriteError::OutOfBound(
                MEMORY_SIZE + 1
            )))
        );
    }

    #[test]
    fn test_write_address_reserved() {
        let mut mem = create_memory();

        let res = mem.store(&[1u8], 0x000);
        assert_eq!(
            res,
            Err(MemoryError::WriteError(WriteError::ReservedAddr(0x000)))
        );

        let res = mem.store(&[1u8], 0x1FF);
        assert_eq!(
            res,
            Err(MemoryError::WriteError(WriteError::ReservedAddr(0x1FF)))
        );
    }
}
