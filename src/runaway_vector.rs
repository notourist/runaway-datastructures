use crate::access::Accessible;
use crate::query::Query::{Access, Rank, Select};
use crate::query::{Query, QueryResult};
use crate::rank::Rankable;
use crate::select::Selectable;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;

const L0_BIT_SIZE: usize = 1 << 32;
const L1_BIT_SIZE: usize = 2048;
const L2_BIT_SIZE: usize = 512;

const L1_INDEX_BIT_SIZE: usize = 32;
const L2_INDEX_BIT_SIZE: usize = 10;

#[derive(Debug)]
struct InterleavedIndex(u64, usize);

impl InterleavedIndex {
    pub fn new(l1: u32, l2s: &[u16]) -> Self {
        let mut value = l1 as u64;

        for (i, l2) in l2s.iter().enumerate() {
            value |= (*l2 as u64) << L2_INDEX_BIT_SIZE * i + L1_INDEX_BIT_SIZE
        }
        InterleavedIndex(value, l2s.len())
    }

    fn l1(&self) -> u32 {
        self.0 as u32
    }

    fn index(&self, index: usize) -> u16 {
        (self.0 >> ((L2_INDEX_BIT_SIZE * index) + L1_INDEX_BIT_SIZE) & (1 << L2_INDEX_BIT_SIZE) - 1)
            as u16
    }

    fn len(&self) -> usize {
        self.1
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
        // FIXME l1 reset
        for (i, chunk) in bit_vec.chunks(L2_BIT_SIZE).enumerate() {
            if i % (L0_BIT_SIZE / L2_BIT_SIZE) == 0 {
                l0_indices.push(l1 as u64 + l0_indices.last().unwrap_or(&0_u64));
                l1 = 0;
            }
            let l2_pos = i % 4;
            let l2 = chunk.count_ones() as u16;
            l2s[l2_pos] = l2;
            if i % (L1_BIT_SIZE / L2_BIT_SIZE) == 3 {
                l12_indices.push(InterleavedIndex::new(l1, &l2s[0..3]));
                l1 = l1 + l2s[0] as u32 + l2s[1] as u32 + l2s[2] as u32 + l2s[3] as u32;
                l2s = [0; 4];
            }
        }
        // Small vector fix
        if bit_vec.len() < 1025 {
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
    fn select0(&self, _nth: usize) -> Option<usize> {
        None
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        assert!(nth > 0);
        assert!(nth <= self.bit_vec.len());
        let mut rank = nth;
        let mut l0_index = 0;
        for (i, l0) in self.l0_indices.iter().enumerate() {
            if *l0 as usize >= nth {
                rank -= *l0 as usize;
                l0_index = i - 1;
                break;
            }
        }
        if rank == 0 {
            return Some(l0_index * L0_BIT_SIZE);
        }
        let mut last_l1 = (l0_index + 1) * (L0_BIT_SIZE / L1_BIT_SIZE) - 1;
        if last_l1 > self.l12_indices.len() - 1 {
            last_l1 = self.l12_indices.len() - 1;
        }
        let mut l = 0;
        let mut r = last_l1;
        let mut result = None;
        while l <= r {
            let m = (l + r) / 2;
            if m == last_l1 {
                result = Some(m);
                break;
            }
            if (self.l12_indices[m].l1() as usize) < rank
                && (self.l12_indices[m + 1].l1() as usize) < rank
            {
                l = m + 1;
            } else if (self.l12_indices[m].l1() as usize) >= rank
                && (self.l12_indices[m + 1].l1() as usize) > rank
            {
                r = m - 1;
            } else {
                result = Some(m);
                break;
            }
        }
        if result == None {
            return None;
        }
        let l1_index = result.unwrap();
        let l12_index = &self.l12_indices[l1_index];
        rank -= l12_index.l1() as usize;
        if rank == 0 {
            return Some(l0_index * L0_BIT_SIZE + l1_index * L1_BIT_SIZE);
        }
        let mut l2_index = 0;
        for i in 0..l12_index.len() {
            if rank > l12_index.index(i) as usize {
                rank -= l12_index.index(i) as usize;
                l2_index += 1;
            } else {
                break;
            }
        }

        if rank == 0 {
            return Some(l0_index * L0_BIT_SIZE + l1_index * L1_BIT_SIZE + l2_index * L2_BIT_SIZE);
        }
        let bit_search_start =
            l0_index * L0_BIT_SIZE + l1_index * L1_BIT_SIZE + (l2_index) * L2_BIT_SIZE;
        let bit_search_end = if bit_search_start + L2_BIT_SIZE > self.bit_vec.len() {
            self.bit_vec.len()
        } else {
            bit_search_start + L2_BIT_SIZE
        };
        if bit_search_start >= bit_search_end {
            return None;
        }
        let mut bit = 0;
        for i in bit_search_start..bit_search_end {
            if self.bit_vec[i] {
                rank -= self.bit_vec[i] as usize;
                if rank == 0 {
                    bit = i;
                    break;
                }
            }
        }
        return Some(
            l0_index * L0_BIT_SIZE
                + l1_index * L1_BIT_SIZE
                + l2_index * L2_BIT_SIZE
                + (bit % L2_BIT_SIZE),
        );
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
    use crate::rank::BlockVector;
    use crate::select::NaiveSelect;
    use bitvec::bitvec;
    use bitvec::field::BitField;
    use bitvec::order::Lsb0;
    use rand::Rng;

    #[test]
    fn rank() {
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
        let assumed_to_be_correct = BlockVector::new(&rand_bv);
        let test_me = RunawayVector::new(&rand_bv);
        for i in 2usize.pow(32) + 1..BIT_VEC_LEN {
            //assert_eq!(test_me.rank0(i), assumed_to_be_correct.rank0(i));
            assert_eq!(test_me.rank1(i), assumed_to_be_correct.rank1(i));
        }
    }

    #[test]
    fn select() {
        const BIT_VEC_LEN: usize = 2usize.pow(20);
        let mut rng = rand::thread_rng();
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        const BITS_PER_RNG_READ: usize = 64;
        let mut rng_i = 0;
        while rng_i < BIT_VEC_LEN / BITS_PER_RNG_READ {
            let num: u64 = rng.gen();
            rand_bv[(rng_i * BITS_PER_RNG_READ)..((rng_i + 1) * BITS_PER_RNG_READ)].store(num);
            rng_i += 1;
        }
        let assumed_to_be_correct = NaiveSelect::new(&rand_bv);
        let test_me = RunawayVector::new(&rand_bv);
        for i in 1..BIT_VEC_LEN {
            //assert_eq!(test_me.rank0(i), assumed_to_be_correct.rank0(i));
            assert_eq!(test_me.select1(i), assumed_to_be_correct.select1(i));
        }
    }
}
