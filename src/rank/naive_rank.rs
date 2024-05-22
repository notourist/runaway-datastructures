use bitvec::prelude as bv;

use crate::rank::Rankable;

pub struct NaiveRank<'a> {
    pub bit_vec: &'a bv::BitSlice<u64, bv::Lsb0>
}

impl Rankable for NaiveRank<'_> {
    fn rank_0(&self, idx: usize) -> usize {
        let mut count = 0;
        for i in 0..idx {
            if !self.bit_vec[i] {
                count += 1;
            }
        }
        count
    }
}