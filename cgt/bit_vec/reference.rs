use crate::ref_wrapper::impl_ref_wrapper;

impl_ref_wrapper! {
    /// Reference to a bit vec
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BitVecRef {
        data: [u8]
    }
}

impl std::fmt::Debug for BitVecRef {
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

impl BitVecRef {
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
