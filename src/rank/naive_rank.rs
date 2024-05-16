use bitvec::prelude as bv;

use crate::rank::Rankable;

pub struct RankNaive<'a> {
    bit_vec: &'a bv::BitVec<usize, bv::Lsb0>
}

impl Rankable for RankNaive<'_> {
    fn rank_1(&self, idx: usize) -> usize {
        let mut count = 0;
        for i in 0..idx {
            if self.bit_vec[i] == true {
                count += 1;
            }
        }
        count
    }
}