mod block_full_no_lookup_rank;
mod interleaved_rank;
mod naive_rank;

pub use block_full_no_lookup_rank::BlockStaticIncrementRank;
pub use interleaved_rank::InterleavedRank;
pub use naive_rank::NaiveRank;

pub trait Rankable {
    fn rank_0(&self, idx: usize) -> usize {
        idx - self.rank_1(idx)
    }
    fn rank_1(&self, idx: usize) -> usize {
        idx - self.rank_0(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::bitvec;
    use bitvec::field::BitField;
    use bitvec::order::Lsb0;
    use rand::Rng;

    #[test]
    fn compare() {
        const BIT_VEC_LEN: usize = 2usize.pow(32);
        let mut rng = rand::thread_rng();
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        const BITS_PER_RNG_READ: usize = 64;
        let mut rng_i = 0;
        while rng_i < BIT_VEC_LEN / BITS_PER_RNG_READ {
            let num: u64 = rng.gen();
            rand_bv[(rng_i * BITS_PER_RNG_READ)..((rng_i + 1) * BITS_PER_RNG_READ)].store(num);
            rng_i += 1;
        }
        let assumed_to_be_correct = BlockStaticIncrementRank::new(&rand_bv);
        let lecture_rank = InterleavedRank::new(&rand_bv);
        for i in BIT_VEC_LEN - 2usize.pow(26)..BIT_VEC_LEN {
            assert_eq!(lecture_rank.rank_0(i), assumed_to_be_correct.rank_0(i));
            assert_eq!(lecture_rank.rank_1(i), assumed_to_be_correct.rank_1(i));
        }
    }
}
