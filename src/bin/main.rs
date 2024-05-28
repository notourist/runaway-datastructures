extern crate rand;
extern crate runaway_datastructures;

use bitvec::bitvec;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use runaway_datastructures::runaway_vector::RunawayVector;
use runaway_datastructures::select::Selectable;

pub fn main() {
    const BIT_VEC_LEN: usize = (1usize << 33) + 4069;
    dbg!(BIT_VEC_LEN);

    let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
    const BITS_PER_RNG_READ: usize = 64;
    let mut i = 0;
    /*while i < BIT_VEC_LEN / BITS_PER_RNG_READ {
        let num: u64 = u64::MIN;
        rand_bv[(i * BITS_PER_RNG_READ)..((i + 1) * BITS_PER_RNG_READ)].store(num);
        i += 1;
    }*/
    rand_bv[0..1].store(1);
    rand_bv[2usize.pow(32)..2usize.pow(32) + 1].store(u64::MAX);
    dbg!(1usize << 32);
    let vector = RunawayVector::new(&rand_bv);
    //for idx in 1..=rand_bv.len() {
        //dbg!(idx);
        //dbg!(vector.select0(idx));
    //}
    dbg!(rand_bv[vector.select1(1)]);
    let space = rand_bv.len() + vector.bit_size();
    println!(
        "RESULT name=Nasarek space={} support_space={} overhead={}",
        space,
        vector.bit_size(),
        vector.bit_size() as f64 / space as f64,
    );
}
