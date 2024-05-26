extern crate rand;
extern crate runaway_datastructures;

use bitvec::bitvec;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use rand::Rng;
use runaway_datastructures::rank::{InterleavedRank, Rankable};

pub fn main() {
    const BIT_VEC_LEN: usize = 2usize.pow(24);
    dbg!(BIT_VEC_LEN);

    let mut rng = rand::thread_rng();

    let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
    const BITS_PER_RNG_READ: usize = 32;
    let mut i = 0;
    while i < BIT_VEC_LEN / BITS_PER_RNG_READ {
        let num: u32 = rng.gen();
        rand_bv[(i * BITS_PER_RNG_READ)..((i + 1) * BITS_PER_RNG_READ)].store(num);
        i += 1;
    }
    let rank = InterleavedRank::new(&rand_bv);
    let idx = 2usize.pow(18);
    dbg!(idx);
    dbg!(rank.rank1(idx));
    let space = rand_bv.len() + rank.bit_size();
    println!(
        "RESULT name=Nasarek space={} support_space={} overhead={}",
        space,
        rank.bit_size(),
        rank.bit_size() as f64 / space as f64,
    );
}
