extern crate runaway_datastructures;
extern crate rand;

use bitvec::bitvec;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use rand::Rng;

pub fn main() {
    const BIT_VEC_LEN: usize = 2usize.pow(34); // 2 GB

    let mut rng = rand::thread_rng();

    let access: usize = rng.gen_range(0..123);
    println!("{}", access);

    let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
    let mut i = 0;
    while i < BIT_VEC_LEN / 64 {
        let num: u64 = rng.gen();
        rand_bv[i..i+64].store(num);
        i += 64;
    }
    println!("access now");
    println!("{}", rand_bv[BIT_VEC_LEN - access]);
}