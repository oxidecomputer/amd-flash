
pub enum Error {
}

pub type Result<Q> = core::result::Result<Q, Error>;
pub type Location = u32;

pub trait Flash {
    fn block_size(&self) -> usize;
    fn read_block(&self, location: Location, buffer: &mut [u8]) -> Result<()>;
    fn write_block(&self, location: Location, buffer: &[u8]) -> Result<()>;
    fn erase_block(&self, location: Location) -> Result<()>;
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
