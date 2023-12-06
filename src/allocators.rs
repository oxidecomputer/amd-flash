use crate::block;
use crate::{Error, Result};
use crate::{Location, Range};

pub struct ArenaAllocator<const BSIZE: block::Size> {
    _efh_range: Range<BSIZE>,
    arenas: [Range<BSIZE>; 2],
}

impl<const BSIZE: block::Size> ArenaAllocator<BSIZE> {
    /// Creates a new allocator that will use parts of the given ARENA.
    /// Depending on processor generation, a part of it will be cut out
    /// and not given to the user (since it needs to be at a fixed
    /// spot and also is used by us).
    pub fn try_new(
        efh_start: Location<BSIZE>,
        efh_size: usize,
        mut arena: Range<BSIZE>,
    ) -> Result<Self> {
        assert!(<Location<BSIZE> as Into<usize>>::into(arena.start) == 0_usize);
        // Avoid EFH_BEGINNING..(EFH_BEGINNING + EFH_SIZE)
        let a_size = efh_start.into();
        let a = arena.split_round_up(a_size).ok_or(Error::Size)?;
        assert!(a.end == efh_start);
        let _efh_range = arena.split_round_up(efh_size).ok_or(Error::Size)?;
        Ok(Self { _efh_range, arenas: [a, arena] })
    }
}
impl<const BSIZE: block::Size> crate::Allocator<BSIZE>
    for ArenaAllocator<BSIZE>
{
    /// From the free ranges, take a range of at least SIZE Bytes,
    /// if possible. Otherwise return None.
    fn alloc_round_up(&mut self, size: usize) -> Option<Range<BSIZE>> {
        self.arenas[0]
            .split_round_up(size)
            .or_else(|| self.arenas[1].split_round_up(size))
    }

    fn max_contiguous_capacity(&self) -> usize {
        self.arenas.iter().max_by_key(|&r| r.size()).unwrap().size()
    }
}

#[cfg(test)]
mod allocator_tests {
    use super::*;
    use crate::Allocator;

    const BSIZE: block::Size = block::Size::B4K;

    fn intersect(
        a: &Range<BSIZE>,
        b: &Range<BSIZE>,
    ) -> Option<(Location<BSIZE>, Location<BSIZE>)> {
        let new_beginning =
            Location::from(a.start).max(Location::from(b.start));
        let new_end = Location::from(a.end).min(Location::from(b.end));
        if new_beginning < new_end {
            Some((new_beginning, new_end))
        } else {
            None
        }
    }

    fn allocator() -> ArenaAllocator<BSIZE> {
        let start = Location::try_new(0).unwrap();
        let end = start.add_round_up(0x4_0000).unwrap();
        ArenaAllocator::try_new(
            Location::try_new(0x2_0000).unwrap(),
            0x200,
            Range::new(start, end),
        )
        .unwrap()
    }
    fn efh_range() -> Range<BSIZE> {
        Range::new(
            Location::try_new(0x2_0000).unwrap(),
            Location::try_new(0x2_0000).unwrap().add_round_up(0x200).unwrap(),
        )
    }

    #[test]
    fn test_allocator_1() {
        let mut alloc = allocator();
        let efh_range = efh_range();
        let a = alloc.alloc_round_up(42).unwrap();
        let b = alloc.alloc_round_up(100).unwrap();
        assert!(intersect(&a, &b).is_none());
        assert!(intersect(&a, &efh_range).is_none());
        assert!(intersect(&b, &efh_range).is_none());
        assert!(<Location<BSIZE> as Into<usize>>::into(b.end) < 0x2_0000);
    }

    #[test]
    fn test_allocator_2() {
        let mut alloc = allocator();
        let efh_range = efh_range();
        let a = alloc.alloc_round_up(0x2_0000).unwrap();
        let b = alloc.alloc_round_up(100).unwrap();
        assert!(intersect(&a, &b).is_none());
        assert!(intersect(&a, &efh_range).is_none());
        assert!(intersect(&b, &efh_range).is_none());
        assert!(<Location<BSIZE> as Into<usize>>::into(b.end) < 0x4_0000);
    }

    #[test]
    fn test_allocator_3() {
        let mut alloc = allocator();
        let efh_range = efh_range();
        let a = alloc.alloc_round_up(0x1_fff8).unwrap();
        let b = alloc.alloc_round_up(100).unwrap();
        assert!(intersect(&a, &b).is_none());
        assert!(intersect(&a, &efh_range).is_none());
        assert!(intersect(&b, &efh_range).is_none());
        assert!(<Location<BSIZE> as Into<usize>>::into(b.end) < 0x4_0000);
        assert!(<Location<BSIZE> as Into<usize>>::into(b.start) > 0x2_0000);
    }
}
