extern crate runaway_datastructures;
extern crate rand;

use bitvec::{bitvec};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use rand::Rng;
use runaway_datastructures::rank::{LectureRank, Rankable};

pub fn main() {
    const BIT_VEC_LEN: usize = 1024;

    let mut rng = rand::thread_rng();

    //let access: usize = rng.gen_range(0..32);
    //println!("{}", access);

    let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
    const BITS_PER_RNG_READ: usize = 32;
    let mut i = 0;
    while i < BIT_VEC_LEN / BITS_PER_RNG_READ {
        let num: u32 = rng.gen();
        rand_bv[(i*BITS_PER_RNG_READ)..((i + 1)*BITS_PER_RNG_READ)].store(num);
        i += 1;
    }
    let lecture_rank = LectureRank::new(&rand_bv);
    println!("{:?}", lecture_rank.rank_0(1021));
}
