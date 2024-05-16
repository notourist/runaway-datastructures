use bitvec::prelude as bv;

pub trait Rankable {
    fn rank_0(&self, idx: usize) -> usize {
        idx - self.rank_1(idx)
    }
    fn rank_1(&self, idx: usize) -> usize;
}

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
