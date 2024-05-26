extern crate rand;
extern crate runaway_datastructures;

use bitvec::bitvec;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use rand::Rng;
use runaway_datastructures::rank::Rankable;
use runaway_datastructures::runaway_vector::RunawayVector;
use runaway_datastructures::select::Selectable;

pub fn main() {
    const BIT_VEC_LEN: usize = 2usize.pow(10);
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
    let vector = RunawayVector::new(&rand_bv);
    for idx in 0..256 {
        dbg!(vector.select1(idx));
    }
    let space = rand_bv.len() + vector.bit_size();
    println!(
        "RESULT name=Nasarek space={} support_space={} overhead={}",
        space,
        vector.bit_size(),
        vector.bit_size() as f64 / space as f64,
    );
}
