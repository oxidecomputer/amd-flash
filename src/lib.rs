// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

use core::convert::TryInto;

pub mod allocators;
pub mod block;

#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    #[cfg_attr(feature = "std", error("io"))]
    Io,
    #[cfg_attr(feature = "std", error("block not aligned for erase"))]
    Alignment,
    #[cfg_attr(feature = "std", error("programmer violated an invariant"))]
    Programmer,
    #[cfg_attr(feature = "std", error("requested size is unavailable"))]
    Size,
    #[cfg_attr(feature = "std", error("overflow"))]
    Overflow,
}

pub type Result<T> = core::result::Result<T, Error>;

/// A flash location that is aligned on a block boundary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Location<const BSIZE: block::Size>(u32);

impl<const BSIZE: block::Size> Location<BSIZE> {
    pub fn try_new(loc: u32) -> Result<Location<BSIZE>> {
        if !BSIZE.is_aligned(loc.try_into().unwrap()) {
            return Err(Error::Alignment);
        }
        Ok(Location(loc))
    }

    pub fn add(&self, amount: usize) -> Result<Self> {
        if !BSIZE.is_aligned(amount) {
            return Err(Error::Alignment);
        }
        let amount: u32 = amount.try_into().map_err(|_| Error::Overflow)?;
        let location: u32 = self.0;
        let pos = location.checked_add(amount).ok_or(Error::Overflow)?;
        Location::try_new(pos)
    }

    /// Round up to a multiple of block size.
    pub fn add_round_up(&self, amount: usize) -> Result<Self> {
        let amount = BSIZE.align_up(amount).ok_or(Error::Alignment)?;
        self.add(amount)
    }

    /// Note: Assumed start <= end, otherwise result will be 0.
    pub fn diff(start: Self, end: Self) -> u32 {
        end.0.saturating_sub(start.0)
    }
}

impl<const BSIZE: block::Size> From<Location<BSIZE>> for u32 {
    fn from(val: Location<BSIZE>) -> Self {
        val.0
    }
}

impl<const BSIZE: block::Size> From<Location<BSIZE>> for usize {
    fn from(val: Location<BSIZE>) -> Self {
        val.0.try_into().unwrap()
    }
}

pub trait Allocator<const BSIZE: block::Size> {
    fn alloc_round_up(&mut self, size: usize) -> Option<Range<BSIZE>>;
    fn max_contiguous_capacity(&self) -> usize;
}

/// A marker trait that signifies that a type is aligned on some boundary.
pub trait Aligned<const BSIZE: block::Size> {}

pub trait Read<const BSIZE: block::Size> {
    /// Read exactly the right amount from the location to fill the
    /// entire BUFFER that was passed.
    fn read_exact(&self, loc: Location<BSIZE>, buf: &mut [u8]) -> Result<()>;

    /// Reads a block of data.
    fn read_block(&self, loc: Location<BSIZE>, buf: &mut [u8]) -> Result<()> {
        if buf.len() != usize::from(BSIZE) {
            return Err(Error::Programmer);
        }
        self.read_exact(loc, buf)
    }
}

pub trait Write<const BSIZE: block::Size>: Read<BSIZE> {
    /// Erase a block at the given location.
    fn erase(&self, loc: Location<BSIZE>) -> Result<()>;

    /// Writes a block, erasing the remainder if the given data
    /// are shorter than a block.
    fn write_block(&self, location: Location<BSIZE>, buf: &[u8]) -> Result<()>;

    /// Writes data into contiguous blocks starting at the given
    /// location.
    fn write(&self, mut location: Location<BSIZE>, buf: &[u8]) -> Result<()> {
        let bsize = usize::from(BSIZE);
        for chunk in buf.chunks(bsize) {
            self.write_block(location, chunk)?;
            if chunk.len() != bsize {
                // TODO: Only allow on last chunk
                break;
            }
            location = location.add(bsize)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Range<const B: block::Size> {
    pub start: Location<B>,
    pub end: Location<B>,
}

impl<const B: block::Size> Range<B> {
    /// Creates a new range of flash locations covering between [start, end).
    pub fn new(start: Location<B>, end: Location<B>) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    /// Splits the Range after at least SIZE Byte, if possible.
    /// Return the first part. Retain the second part.
    pub fn split_round_up(&mut self, size: usize) -> Option<Self> {
        let start = self.start;
        let end = self.start.add_round_up(size).ok()?;
        if end > self.end {
            return None;
        }
        *self = Self::new(end, self.end);
        Some(Self::new(start, end))
    }

    /// Returns the size of the range in bytes.
    pub fn size(&self) -> usize {
        Location::diff(self.start, self.end) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    const BSIZE: block::Size = block::Size::B64K;
    const BLOCK_SIZE: usize = BSIZE as usize;

    struct FlashImage<'a> {
        buf: RefCell<&'a mut [u8]>,
    }

    impl<'a> FlashImage<'a> {
        pub fn new(buf: &'a mut [u8]) -> Self {
            Self { buf: RefCell::new(buf) }
        }
    }

    impl Read<BSIZE> for FlashImage<'_> {
        fn read_exact(
            &self,
            loc: Location<BSIZE>,
            dst: &mut [u8],
        ) -> Result<()> {
            let src = self.buf.borrow();
            let loc = usize::from(loc);
            let block = &src[loc..loc + dst.len()];
            dst.copy_from_slice(block);
            Ok(())
        }
    }

    impl Write<BSIZE> for FlashImage<'_> {
        fn erase(&self, loc: Location<BSIZE>) -> Result<()> {
            let mut buf = self.buf.borrow_mut();
            let loc = usize::from(loc);
            let block = &mut buf[loc..loc + usize::from(BSIZE)];
            block.fill(0xff);
            Ok(())
        }

        fn write_block(&self, loc: Location<BSIZE>, src: &[u8]) -> Result<()> {
            let mut dst = self.buf.borrow_mut();
            let loc = usize::from(loc);
            let block = &mut dst[loc..loc + usize::from(BSIZE)];
            block.copy_from_slice(src);
            Ok(())
        }
    }

    #[test]
    fn flash_image_usage() -> Result<()> {
        let mut storage = [0xFFu8; BLOCK_SIZE * 2];
        let image = FlashImage::new(&mut storage[..]);
        let beginning_1 = Location::try_new(0).unwrap();
        image.write_block(beginning_1, &[1u8; BLOCK_SIZE])?;
        let beginning_2 = Location::try_new(BLOCK_SIZE as u32).unwrap();
        image.write_block(beginning_2, &[2u8; BLOCK_SIZE])?;
        let mut buf: [u8; BLOCK_SIZE] = [0u8; BLOCK_SIZE];
        image.read_exact(Location::try_new(0).unwrap(), &mut buf)?;
        assert_eq!(buf, [1u8; BLOCK_SIZE]);
        image.read_exact(
            Location::try_new(BLOCK_SIZE as u32).unwrap(),
            &mut buf,
        )?;
        assert_eq!(buf, [2u8; BLOCK_SIZE]);
        Ok(())
    }
}
