//! Fixed-size bit vec

/// Fixed-size bit vec
///
/// Note that the size is in *bytes* rather than bits since that would require `generic_const_exprs`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedBitVec<const BYTE_LEN: usize> {
    data: [u8; BYTE_LEN],
}

impl<const BYTE_LEN: usize> std::fmt::Debug for FixedBitVec<BYTE_LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::io::Cursor;
        use std::io::Write;

        let mut list = f.debug_list();

        // HACK: entry_with is unstable
        for byte in &self.data {
            let mut buf = [0u8; 16];
            let mut cursor = Cursor::new(&mut buf[..]);
            write!(cursor, "0b{byte:08b}").unwrap();
            let len = cursor.position();
            let buf = cursor.into_inner();
            let buf = &buf[0..len as usize];
            // SAFETY: We just wrote to buf
            let s = unsafe { str::from_utf8_unchecked(buf) };
            list.entry(&s);
        }

        list.finish()
    }
}

impl<const BYTE_LEN: usize> FixedBitVec<BYTE_LEN> {
    /// Construct new empty (all false) bitvec
    #[inline]
    pub const fn new() -> FixedBitVec<BYTE_LEN> {
        FixedBitVec {
            data: [0; BYTE_LEN],
        }
    }

    /// Construct new filled (all true) bitvec
    #[inline]
    pub const fn filled() -> FixedBitVec<BYTE_LEN> {
        FixedBitVec {
            data: [u8::MAX; BYTE_LEN],
        }
    }

    /// Get value at given *bit* index
    ///
    /// # Panics
    ///
    /// When index is out of bounds
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let byte_index = index >> 3;
        let bit_mask = 1 << (index & 0b111);
        let byte = self.data[byte_index];
        (byte & bit_mask) != 0
    }

    /// Set value at given *bit* index
    ///
    /// # Panics
    ///
    /// When index is out of bounds
    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let byte_index = index >> 3;
        let bit_index = index & 0b111;
        self.data[byte_index] &= !(1 << bit_index);
        self.data[byte_index] |= (value as u8) << bit_index;
    }
}

#[test]
fn get_set() {
    let mut bit_vec = FixedBitVec::<{ 32 / 8 }>::new();
    bit_vec.set(10, true);
    bit_vec.set(16, true);
    assert_eq!(bit_vec.get(4), false);
    assert_eq!(bit_vec.get(10), true);
    assert_eq!(bit_vec.get(14), false);
    assert_eq!(bit_vec.get(16), true);
    assert_eq!(bit_vec.get(30), false);
}
