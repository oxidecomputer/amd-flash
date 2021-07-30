
#[derive(Debug)]
pub enum Error {
    Io,
}

pub type Result<Q> = core::result::Result<Q, Error>;
pub type Location = u32;

pub trait FlashRead<const READING_BLOCK_SIZE: usize> {
    fn read_block(&self, location: Location, buffer: &mut [u8; READING_BLOCK_SIZE]) -> Result<()>;
}

pub trait FlashWrite<const WRITING_BLOCK_SIZE: usize, const ERASURE_BLOCK_SIZE: usize> {
    fn write_block(&mut self, location: Location, buffer: &[u8; WRITING_BLOCK_SIZE]) -> Result<()>;
    fn erase_block(&mut self, location: Location) -> Result<()>;
}

#[cfg(test)]
mod tests {
/*
struct MmioFlash {

}

impl Flash for MmioFlash {
    fn block_size(&self) -> usize {
        1
    }
    fn read_block(&self, location: Location, buffer: &mut [u8]) -> Result<()> {
        Ok(())
    }
    fn write_block(&self, location: Location, buffer: &[u8]) -> Result<()> {
        Ok(())
    }
    fn erase_block(&self, location: Location) -> Result<()> {
        Ok(())
    }
}
*/



    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
