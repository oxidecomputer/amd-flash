
#![no_std]

use core::convert::TryInto;
use core::convert::TryFrom;

#[derive(Debug)]
pub enum Error {
    Io,
    Alignment,
}

pub type Result<Q> = core::result::Result<Q, Error>;

/// This is any Location on the Flash chip
pub type Location = u32;

/// This is a Location which definitely is aligned on an erase block boundary
#[derive(Clone, Copy)]
pub struct ErasableLocation<const ERASABLE_BLOCK_SIZE: usize>(Location);

// FIXME: (fyi, arg2.saturating_sub(arg1) does that if you hadn't found it)

impl<const ERASABLE_BLOCK_SIZE: usize> ErasableLocation<ERASABLE_BLOCK_SIZE> {
    /// Note: Assumed beginning <= self, otherwise result will be 0.
    pub fn extent(beginning: Self, end: Self) -> u32 {
        let beginning = beginning.0;
        let end = end.0;
        end.saturating_sub(beginning)
    }
    pub fn advance(&self, amount: usize) -> Result<Self> {
        if amount % ERASABLE_BLOCK_SIZE == 0 {
            let pos = (self.0 as usize).checked_add(amount).ok_or(Error::Alignment)?;
            Ok(Self(pos.try_into().map_err(|_| Error::Alignment)?))
        } else {
            Err(Error::Alignment)
        }
    }
}

impl<const ERASABLE_BLOCK_SIZE: usize> TryFrom<Location> for ErasableLocation<ERASABLE_BLOCK_SIZE> {
    type Error = Error;

    fn try_from(value: Location) -> Result<Self> {
        if (value as usize) % ERASABLE_BLOCK_SIZE == 0 {
            Ok(Self(value))
        } else {
            Err(Error::Alignment)
        }
    }
}

impl<const ERASABLE_BLOCK_SIZE: usize> From<ErasableLocation<ERASABLE_BLOCK_SIZE>> for Location {
    fn from(source: ErasableLocation<ERASABLE_BLOCK_SIZE>) -> Self {
        source.0
    }
}

pub trait FlashRead<const ERASABLE_BLOCK_SIZE: usize> {
    fn read_exact(&self, location: Location, buffer: &mut [u8]) -> Result<usize>;
    fn read_erasable_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>, buffer: &mut [u8; ERASABLE_BLOCK_SIZE]) -> Result<()>;
}

pub trait FlashWrite<const ERASABLE_BLOCK_SIZE: usize> {
    fn erase_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>) -> Result<()>;
    fn erase_and_write_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>, buffer: &[u8; ERASABLE_BLOCK_SIZE]) -> Result<()>;
    fn grow_to_erasable_block(&self, beginning: Location, end: Location) -> (Location, Location) {
        let erasable_block_size: u32 = ERASABLE_BLOCK_SIZE.try_into().unwrap();
        let beginning_misalignment = beginning % erasable_block_size;
        let end_misalignment = if end % erasable_block_size == 0 {
            0
        } else {
            erasable_block_size - (end % erasable_block_size)
        };
        let beginning = beginning.saturating_sub(beginning_misalignment);
        let end = end.checked_add(end_misalignment).unwrap();
        (beginning, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    struct FlashImage<'a> {
        buf: Cell<&'a mut [u8]>,
    }

    impl<'a> FlashImage<'a> {
        pub fn new(buf: &'a mut [u8]) -> Self {
            Self {
                buf: Cell::new(buf)
            }
        }
    }

    const ERASABLE_BLOCK_SIZE: usize = 0x2_0000;
    impl FlashRead<ERASABLE_BLOCK_SIZE> for FlashImage<'_> {
        fn read_exact(&self, location: Location, buffer: &mut [u8]) -> Result<usize> {
            let len = buffer.len();
            let buf = self.buf.get();
            let block = &buf[location as usize ..];
            let block = block[0..len];
            buffer[..].copy_from_slice(block);
            Ok(len)
        }
        fn read_erasable_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>, buffer: &mut [u8; ERASABLE_BLOCK_SIZE]) -> Result<()> {
            let location: Location = location.into();
            let buf = self.buf.get();
            let block = &buf[location as usize .. (location as usize + ERASABLE_BLOCK_SIZE)];
            buffer[..].copy_from_slice(block);
            Ok(())
        }
    }

    impl FlashWrite<ERASABLE_BLOCK_SIZE> for FlashImage<'_> {
        fn erase_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>) -> Result<()> {
            let location: Location = location.into();
            let mut buf = self.buf.get().clone();
            let block = &mut buf[location as usize .. (location as usize + ERASABLE_BLOCK_SIZE)];
            for e in block.iter_mut() {
                *e = 0xFF;
            }
            self.buf.set(buf);
            Ok(())
        }
        fn erase_and_write_block(&self, location: ErasableLocation<ERASABLE_BLOCK_SIZE>, buffer: &[u8; ERASABLE_BLOCK_SIZE]) -> Result<()> {
            let location: Location = location.into();
            let mut buf = self.buf.get().clone();
            let block = &mut buf[location as usize .. (location as usize + ERASABLE_BLOCK_SIZE)];
            block.copy_from_slice(&buffer[..]);
            self.buf.set(buf);
            Ok(())
        }
    }

    #[test]
    fn flash_image_usage() -> Result<()> {
        let mut storage = vec![0xFF; 0x100_0000];
        let mut flash_image = FlashImage::new(&mut storage[..]);
        flash_image.erase_and_write_block(ErasableLocation::<ERASABLE_BLOCK_SIZE>::try_from(Location::from(0)).unwrap(), &[1u8; ERASABLE_BLOCK_SIZE])?;
        flash_image.erase_and_write_block(ErasableLocation::<ERASABLE_BLOCK_SIZE>::try_from(Location::from(ERASABLE_BLOCK_SIZE as u32)).unwrap(), &[2u8; ERASABLE_BLOCK_SIZE])?;
        let mut buf: [u8; ERASABLE_BLOCK_SIZE] = [0u8; ERASABLE_BLOCK_SIZE];
        flash_image.read_exact(0, &mut buf)?;
        assert_eq!(buf, [1u8; ERASABLE_BLOCK_SIZE]);
        flash_image.read_exact(ERASABLE_BLOCK_SIZE as Location, &mut buf)?;
        assert_eq!(buf, [2u8; ERASABLE_BLOCK_SIZE]);
        Ok(())
    }
}
