
use std::convert::TryInto;

#[derive(Debug)]
pub enum Error {
    Io,
}

pub type Result<Q> = core::result::Result<Q, Error>;
pub type Location = u32;

pub trait FlashRead<const READING_BLOCK_SIZE: usize, const ERASURE_BLOCK_SIZE: usize> {
    fn read_block(&self, location: Location, buffer: &mut [u8; READING_BLOCK_SIZE]) -> Result<()>;
    fn read_erasure_block(&self, location: Location, buffer: &mut [u8; ERASURE_BLOCK_SIZE]) -> Result<()>;
}

pub trait FlashWrite<const WRITING_BLOCK_SIZE: usize, const ERASURE_BLOCK_SIZE: usize> {
    fn write_block(&self, location: Location, buffer: &[u8; WRITING_BLOCK_SIZE]) -> Result<()>;
    fn erase_block(&self, location: Location) -> Result<()>;
    fn erase_and_write_block(&self, location: Location, buffer: &[u8; ERASURE_BLOCK_SIZE]) -> Result<()>;
    fn grow_to_erasure_block(beginning: Location, end: Location) -> (Location, Location) {
        let erasure_block_size: u32 = ERASURE_BLOCK_SIZE.try_into().unwrap();
        let beginning_misalignment = beginning % erasure_block_size;
        let end_misalignment = if end % erasure_block_size == 0 {
            0
        } else {
            erasure_block_size - (end % erasure_block_size)
        };
        let beginning = beginning.saturating_sub(beginning_misalignment);
        let end = end.checked_add(end_misalignment).unwrap();
        (beginning, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FlashImage<'a> {
        buf: &'a mut [u8],
    }

    impl<'a> FlashImage<'a> {
        pub fn new(buf: &'a mut [u8]) -> Self {
            Self {
                buf
            }
        }
    }

    impl FlashRead<0x1000, 0x2_0000> for FlashImage<'_> {
        fn read_block(&self, location: Location, buffer: &mut [u8; 0x1000]) -> Result<()> {
            let block = &self.buf[location as usize .. (location as usize + 0x1000)];
            buffer[..].copy_from_slice(block);
            Ok(())
        }
        fn read_erasure_block(&self, location: Location, buffer: &mut [u8; 0x2_0000]) -> Result<()> {
            let block = &self.buf[location as usize .. (location as usize + 0x2_0000)];
            buffer[..].copy_from_slice(block);
            Ok(())
        }
    }

    impl FlashWrite<0x1000, 0x2_0000> for FlashImage<'_> {
        fn write_block(&mut self, location: Location, buffer: &[u8; 0x1000]) -> Result<()> {
            let block = &mut self.buf[location as usize .. (location as usize + 0x1000)];
            block.copy_from_slice(&buffer[..]);
            Ok(())
        }
        fn erase_block(&mut self, location: Location) -> Result<()> {
            let block = &mut self.buf[location as usize .. (location as usize + 0x2_0000)];
            for e in block.iter_mut() {
                *e = 0xFF;
            }
            Ok(())
        }
    }

    #[test]
    fn flash_image_usage() -> Result<()> {
        let mut storage = vec![0xFF; 0x100_0000];
        let mut flash_image = FlashImage::new(&mut storage[..]);
        flash_image.write_block(0, &[1u8; 0x1000])?;
        flash_image.write_block(0x1000, &[2u8; 0x1000])?;
        let mut buf: [u8; 0x1000] = [0u8; 0x1000];
        flash_image.read_block(0, &mut buf)?;
        assert_eq!(buf, [1u8; 0x1000]);
        flash_image.read_block(0x1000, &mut buf)?;
        assert_eq!(buf, [2u8; 0x1000]);
        Ok(())
    }
}
