use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::access::Accessible;
use crate::query::{Query, QueryResult};
use crate::query::Query::{Access, Rank, Select};
use crate::rank::Rankable;
use crate::select::Selectable;

const L0_BIT_SIZE: usize = 1 << 32;
const L1_BIT_SIZE: usize = 2048;
const L2_BIT_SIZE: usize = 512;

const L1_INDEX_BIT_SIZE: usize = 32;
const L2_INDEX_BIT_SIZE: usize = 10;

#[derive(Debug)]
struct InterleavedIndex(u64);

impl InterleavedIndex {
    pub fn new(l1: u32, l2s: &[u16]) -> Self {
        let mut value = l1 as u64;

        for (i, l2) in l2s.iter().enumerate() {
            value |= (*l2 as u64) << L2_INDEX_BIT_SIZE * i + L1_INDEX_BIT_SIZE
        }
        InterleavedIndex(value)
    }

    fn l1(&self) -> u32 {
        self.0 as u32
    }

    fn index(&self, index: usize) -> u16 {
        (self.0 >> ((L2_INDEX_BIT_SIZE * index) + L1_INDEX_BIT_SIZE) & (1 << L2_INDEX_BIT_SIZE) - 1)
            as u16
    }
}



pub struct RunawayVector<'a> {
    bit_vec: &'a BitVec<u64, Lsb0>,
    l12_indices: Vec<InterleavedIndex>,
    l0_indices: Vec<u64>,
}

impl<'a> RunawayVector<'a> {
    pub fn new(bit_vec: &'a BitVec<u64, Lsb0>) -> Self {
        let l0_capacity = (bit_vec.len() / L0_BIT_SIZE) + 1;
        let l12_capacity = (bit_vec.len() / 64) + 1;

        let mut l0_indices: Vec<u64> = Vec::with_capacity(l0_capacity);
        let mut l12_indices: Vec<InterleavedIndex> = Vec::with_capacity(l12_capacity);

        let mut l0: u64 = 0;
        let mut l1: u32 = 0;
        let mut l2s: [u16; 4] = [0; 4];
        for (i, chunk) in bit_vec.chunks(L2_BIT_SIZE).enumerate() {
            if i % (L0_BIT_SIZE / L2_BIT_SIZE) == 0 {
                l0_indices.push(l0);
                l1 = 0;
            }
            let l2_pos = i % 4;
            let l2 = chunk.count_ones() as u16;
            l2s[l2_pos] = l2;
            if i % (L1_BIT_SIZE / L2_BIT_SIZE) == 3 {
                l12_indices.push(InterleavedIndex::new(l1, &l2s[0..3]));
                l1 = l1 + l2s[0] as u32 + l2s[1] as u32 + l2s[2] as u32 + l2s[3] as u32;
                l0 += l1 as u64;
                l2s = [0; 4];
            }
        }
        // Small vector fix
        if l12_indices.len() == 0 {
            l12_indices.push(InterleavedIndex::new(l1, &l2s[0..3]));
        }
        RunawayVector {
            bit_vec,
            l0_indices,
            l12_indices,
        }
    }

    pub fn bit_size(&self) -> usize {
        self.l12_indices.len() * 64 + self.l0_indices.len() * 64
    }

    pub fn process(&self, query: &Query) -> QueryResult {
        match query {
            Access(idx) => QueryResult::Access(self.access(*idx)),
            Rank(w, idx) => QueryResult::Rank(match *w {
                true => self.rank1(*idx),
                false => self.rank0(*idx),
            }),
            Select(w, nth) => QueryResult::Select(match *w {
                true => self.select1(*nth),
                false => self.select0(*nth),
            }),
        }
    }
}

impl<'a> Selectable for RunawayVector<'a> {
    fn select0(&self, nth: usize) -> Option<usize> {
        let mut l0_index = 0;
        for (i, l0) in self.l0_indices.iter().enumerate() {
            if *l0 as usize >= nth {
                l0_index = i;
                break;
            }
        }
        todo!()
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        None
    }
}

impl<'a> Rankable for RunawayVector<'a> {
    fn rank1(&self, idx: usize) -> usize {
        assert!(idx < self.bit_vec.len());
        let l0_pos = idx / L0_BIT_SIZE;
        let l1_pos = idx / L1_BIT_SIZE;
        let l2_pos = (idx / L2_BIT_SIZE) % 4;
        let bit_idx = idx % L2_BIT_SIZE;

        let l0: usize = self.l0_indices[l0_pos] as usize;
        let l12 = &self.l12_indices[l1_pos];
        let l1: usize = l12.l1() as usize;
        let mut l2: usize = 0;
        for i in 0..l2_pos {
            l2 += l12.index(i) as usize;
        }
        let hand_counted = self.bit_vec[idx - bit_idx..idx].count_ones();
        l0 + l1 + l2 + hand_counted
    }
}

impl<'a> Accessible for RunawayVector<'a> {
    fn access(&self, idx: usize) -> bool {
        self.bit_vec[idx]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::bitvec;
    use bitvec::field::BitField;
    use bitvec::order::Lsb0;
    use rand::Rng;
    use crate::rank::BlockRank;

    #[test]
    fn compare() {
        const BIT_VEC_LEN: usize = 2usize.pow(33);
        let mut rng = rand::thread_rng();
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        const BITS_PER_RNG_READ: usize = 64;
        let mut rng_i = 0;
        while rng_i < BIT_VEC_LEN / BITS_PER_RNG_READ {
            let num: u64 = rng.gen();
            rand_bv[(rng_i * BITS_PER_RNG_READ)..((rng_i + 1) * BITS_PER_RNG_READ)].store(num);
            rng_i += 1;
        }
        let assumed_to_be_correct = BlockRank::new(&rand_bv);
        let test_me = RunawayVector::new(&rand_bv);
        for i in 2usize.pow(32) + 1..BIT_VEC_LEN {
            //assert_eq!(test_me.rank0(i), assumed_to_be_correct.rank0(i));
            assert_eq!(test_me.rank1(i), assumed_to_be_correct.rank1(i));
        }
    }
}