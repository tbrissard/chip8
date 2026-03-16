/// Total addressable memory
const MEMORY_SIZE: usize = 4096;
type Address = usize;

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Debug)]
struct Memory {
    mem: [u8; MEMORY_SIZE],
}

impl Memory {
    pub(crate) fn store(&mut self, data: &[u8], start: Address) -> Result<()> {
        todo!()
    }
}

impl std::default::Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0u8; MEMORY_SIZE],
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
    OutOfBoundAddr(Address),
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
            Err(MemoryError::WriteError(WriteError::OutOfBoundAddr(
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
