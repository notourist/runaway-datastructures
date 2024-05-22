mod lecture_rank;
mod naive_rank;

pub use lecture_rank::LectureRank;
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
    use bitvec::bitvec;
    use bitvec::field::BitField;
    use bitvec::order::Lsb0;
    use rand::Rng;
    use super::*;

    const BIT_VEC_LEN: usize = 2usize.pow(22);

    #[test]
    fn compare() {
        let mut rng = rand::thread_rng();
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        const BITS_PER_RNG_READ: usize = 64;
        let mut rng_i = 0;
        while rng_i < BIT_VEC_LEN / BITS_PER_RNG_READ {
            let num: u64 = rng.gen();
            rand_bv[(rng_i * BITS_PER_RNG_READ)..((rng_i + 1) * BITS_PER_RNG_READ)].store(num);
            rng_i += 1;
        }
        let naive_rank = NaiveRank {
            bit_vec: &rand_bv,
        };
        let mut binding = rand_bv.clone();
        let lecture_rank = LectureRank::new(&mut binding);
        for i in 0..BIT_VEC_LEN {
            assert_eq!(lecture_rank.rank_0(i), naive_rank.rank_0(i));
            assert_eq!(lecture_rank.rank_1(i), naive_rank.rank_1(i));
        }
    }
}
