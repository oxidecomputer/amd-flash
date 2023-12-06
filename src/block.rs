use core::marker::ConstParamTy;

const KIB: usize = 1024;

#[derive(Clone, ConstParamTy, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum Size {
    B4K = 4 * KIB,
    B8K = 8 * KIB,
    B12K = 12 * KIB,
    B16K = 16 * KIB,
    B20K = 20 * KIB,
    B24K = 24 * KIB,
    B28K = 28 * KIB,
    B32K = 32 * KIB,
    B36K = 36 * KIB,
    B40K = 40 * KIB,
    B44K = 44 * KIB,
    B48K = 48 * KIB,
    B52K = 52 * KIB,
    B56K = 56 * KIB,
    B60K = 60 * KIB,
    B64K = 64 * KIB,
}

impl Size {
    /// Returns true if the given integer is a multiple of self.
    pub const fn is_aligned(self, n: usize) -> bool {
        matches!(self.align_up(n), Some(nn) if nn == n)
    }

    /// Returns Some(n rounded to the next multiple) of self,
    /// or None on overflow.
    pub const fn align_up(self, n: usize) -> Option<usize> {
        n.checked_next_multiple_of(self as usize)
    }

    /// Tries to convert a number representing the block size
    /// into a Size variant.
    pub const fn try_from_block_size(block_size: u32) -> Option<Size> {
        match block_size {
            0 => Some(Size::B64K),
            1 => Some(Size::B4K),
            2 => Some(Size::B8K),
            3 => Some(Size::B12K),
            4 => Some(Size::B16K),
            5 => Some(Size::B20K),
            6 => Some(Size::B24K),
            7 => Some(Size::B28K),
            8 => Some(Size::B32K),
            9 => Some(Size::B36K),
            10 => Some(Size::B40K),
            11 => Some(Size::B44K),
            12 => Some(Size::B48K),
            13 => Some(Size::B52K),
            14 => Some(Size::B56K),
            15 => Some(Size::B60K),
            _ => None,
        }
    }
}

impl From<Size> for usize {
    /// Conversions from Size to usize are infallible.
    fn from(s: Size) -> usize {
        s as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_aligned() {
        assert!(Size::B4K.is_aligned(0));
        assert!(Size::B4K.is_aligned(4096));
        assert!(Size::B4K.is_aligned(8192));
        assert!(!Size::B4K.is_aligned(1));
    }

    #[test]
    fn align_up() {
        assert_eq!(Size::B4K.align_up(0), Some(0));
        assert_eq!(Size::B4K.align_up(1), Some(4096));
        assert_eq!(Size::B4K.align_up(4096), Some(4096));
    }

    #[test]
    fn from_impl() {
        assert_eq!(4096_usize, Size::B4K.into());
        assert_eq!(8192_usize, Size::B8K.into());
        assert_eq!(12 * KIB, Size::B12K.into());
        assert_eq!(16 * KIB, Size::B16K.into());
        assert_eq!(32 * KIB, Size::B32K.into());
        assert_eq!(64 * KIB, Size::B64K.into());
    }

    #[test]
    fn try_from_block_size() {
        assert_eq!(Size::try_from_block_size(0), Some(Size::B64K));
        for k in 1..16 {
            let size = Size::try_from_block_size(k);
            assert!(size.is_some());
            let n = usize::from(size.unwrap());
            assert_eq!(k as usize * 4 * KIB, n);
        }
        assert!(Size::try_from_block_size(16).is_none());
    }
}
