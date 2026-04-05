use crate::screen::DIGITS;

pub(crate) type Address = u16;
const MEMORY_SIZE: Address = 0xFFF;

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Debug)]
pub(crate) struct Memory {
    data: [u8; MEMORY_SIZE as usize],
}

const DIGITS_ADDR: Address = 0x0;

pub(crate) fn digit_addr(value: u8) -> Address {
    DIGITS_ADDR + 5 * value as Address
}

impl Default for Memory {
    fn default() -> Self {
        let mut data = [0u8; MEMORY_SIZE as usize];

        data[DIGITS_ADDR as usize..DIGITS.as_flattened().len()]
            .copy_from_slice(DIGITS.as_flattened());

        Self { data }
    }
}

impl Memory {
    #[allow(clippy::absurd_extreme_comparisons)]
    fn is_reserved(addr: Address) -> bool {
        const RESERVED_SPACE_START: Address = 0x000;
        const RESERVED_SPACE_END: Address = 0x1FF;
        (RESERVED_SPACE_START..=RESERVED_SPACE_END).contains(&addr)
    }

    /// Write the bytes from bytes at addr
    ///
    /// In memory, the first byte of each instruction should be located at an even addresses. If a program includes sprite data, it should be padded so any instructions following it will be properly situated in RAM.
    pub(crate) fn store(&mut self, bytes: &[u8], addr: Address) -> Result<()> {
        if Memory::is_reserved(addr) {
            return Err(MemoryError::ReservedAddr(AccessKind::Write, addr));
        }

        if addr as usize + bytes.len() - 1 > MEMORY_SIZE as usize {
            return Err(MemoryError::OutOfBound(AccessKind::Write, addr));
        }

        for (b, a) in bytes.iter().zip(addr..) {
            self.data[a as usize] = *b;
        }

        Ok(())
    }

    pub(crate) fn read(&mut self, addr: Address, n: Address) -> Result<&[u8]> {
        // if Memory::is_reserved(addr) {
        //     return Err(ReadError::ReservedAddr(addr).into());
        // }

        if addr + n - 1 > MEMORY_SIZE {
            return Err(MemoryError::OutOfBound(AccessKind::Read, addr));
        }

        Ok(&self.data[addr as usize..(addr + n) as usize])
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MemoryError {
    #[error("cannot {0:?}: address is out of bound ({1:<#05X})")]
    OutOfBound(AccessKind, Address),
    #[error("cannot {0:?}: address is reserved ({1:<#05X})")]
    ReservedAddr(AccessKind, Address),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AccessKind {
    Write,
    Read,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA: [u8; 3] = [1, 2, 3];
    const TEST_ADDR: Address = 0x200;

    fn create_memory() -> Memory {
        Memory::default()
    }

    #[test]
    fn store_at_start() {
        let mut mem = create_memory();
        mem.store(&TEST_DATA, TEST_ADDR).unwrap();
        assert_eq!(
            &mem.data[TEST_ADDR as usize..TEST_ADDR as usize + TEST_DATA.len()],
            &TEST_DATA
        );
    }

    #[test]
    fn store_out_of_bounds() {
        let mut mem = create_memory();
        let res = mem.store(&[1u8], MEMORY_SIZE + 1);
        assert_eq!(
            res,
            Err(MemoryError::OutOfBound(AccessKind::Write, MEMORY_SIZE + 1))
        );
    }

    #[test]
    fn store_reserved() {
        let mut mem = create_memory();

        let res = mem.store(&[1u8], 0x000);
        assert_eq!(
            res,
            Err(MemoryError::ReservedAddr(AccessKind::Write, 0x000))
        );

        let res = mem.store(&[1u8], 0x1FF);
        assert_eq!(
            res,
            Err(MemoryError::ReservedAddr(AccessKind::Write, 0x1FF))
        );
    }

    #[test]
    fn read_at_start() {
        let mut mem = create_memory();

        mem.data[TEST_ADDR as usize] = TEST_DATA[0];
        mem.data[TEST_ADDR as usize + 1] = TEST_DATA[1];
        mem.data[TEST_ADDR as usize + 2] = TEST_DATA[2];

        assert_eq!(
            mem.read(TEST_ADDR, TEST_DATA.len() as Address).unwrap(),
            TEST_DATA
        );
    }
}
