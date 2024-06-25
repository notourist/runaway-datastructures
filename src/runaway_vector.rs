//! # RunawayVector
//!
//! This succinct bit vector with rank and select support was originally presented by
//! [Zhou et al.](https://doi.org/10.1007/978-3-642-38527-8) It has a space overhead of `o(n)` and
//! answers rank and select queries in `O(1)`.
//!
//! # Internal structure and space usage
//! This bit vector uses three indices
//!
//! # Initialization
//!
//! # Rank
//!
//! # Select
//!
use crate::query::Query::{Access, Rank, Select};
use crate::query::{Query, QueryResult};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use std::{cmp, mem};

const L0_BIT_SIZE: usize = 1 << 32;
const L1_BIT_SIZE: usize = 2048;
const L2_BIT_SIZE: usize = 512;

const L1_INDEX_BIT_SIZE: usize = 32;
const L2_INDEX_BIT_SIZE: usize = 10;

const L1_IN_L0_COUNT: usize = L0_BIT_SIZE / L1_BIT_SIZE;

#[derive(Debug)]
struct InterleavedIndex(u64, usize);

impl InterleavedIndex {
    pub fn new(l1: u32, l2s: &[u16]) -> Self {
        assert!(l2s.len() < 4);
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
        assert!(index < self.1);
        (self.0 >> ((L2_INDEX_BIT_SIZE * index) + L1_INDEX_BIT_SIZE) & (1 << L2_INDEX_BIT_SIZE) - 1)
            as u16
    }

    fn len(&self) -> usize {
        self.1
    }
}

/// A succinct bit vector which supports rank and select queries in `O(1)` with a space usage in
/// `o(n)`.
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
        let mut l2_len = 0;
        // Iterate over each L2 block
        for (i, chunk) in bit_vec.chunks(L2_BIT_SIZE).enumerate() {
            let l2_pos = i % 4;
            l2s[l2_pos] = chunk.count_ones() as u16;
            l2_len += 1;
            // We are at the last l2 block and need to update the interleaved index.
            if i % (L1_BIT_SIZE / L2_BIT_SIZE) == 3 {
                l12_indices.push(InterleavedIndex::new(l1, &l2s[0..l2_len - 1]));
                l1 = l1 + l2s[0] as u32 + l2s[1] as u32 + l2s[2] as u32 + l2s[3] as u32;
                l2s = [0; 4];
                l2_len = 0;
            }
            // We are at the end of the L0 block and need to append the L0 bit count
            if i % (L0_BIT_SIZE / L2_BIT_SIZE) == (L0_BIT_SIZE / L2_BIT_SIZE) - 1 {
                l0_indices.push(l0);
                l0 += l1 as u64;
                l1 = 0;
            }
        }
        // Fix, if the vector length < L1 block length
        if bit_vec.len() % L1_BIT_SIZE != 0 {
            l12_indices.push(InterleavedIndex::new(l1, &l2s[0..l2_len]));
        }
        // Fix, if the vector length < L0 block length
        if bit_vec.len() % L0_BIT_SIZE != 0 {
            l0_indices.push(l0);
        }
        RunawayVector {
            bit_vec,
            l0_indices,
            l12_indices,
        }
    }

    pub fn process(&self, query: &Query) -> QueryResult {
        match query {
            Access(idx) => QueryResult::Access(self.bit_vec[*idx]),
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

    pub fn select0(&self, mut rank: usize) -> Option<usize> {
        assert!(rank > 0);
        assert!(rank <= self.bit_vec.len());
        let mut l0_pos = 0;
        // As the indices store the amount of ones we need to subtract each index from
        // the total amount of bits ot get the amount of zeros inside a block.
        while l0_pos + 1 < self.l0_indices.len()
            && ((l0_pos + 1) * L0_BIT_SIZE - self.l0_indices[l0_pos + 1] as usize) < rank
        {
            l0_pos += 1;
        }
        // The first L0 index is always zero. In this case we cannot subtract the amount of L0 indices
        // from the amount of ones which came before this index because there are none.
        if l0_pos != 0 {
            rank -= l0_pos * L0_BIT_SIZE - self.l0_indices[l0_pos] as usize;
        }
        let first_l1 = l0_pos * L1_IN_L0_COUNT;
        let last_l1 = cmp::min((l0_pos + 1) * L1_IN_L0_COUNT, self.l12_indices.len() - 1);
        let mut l = first_l1;
        let mut r = last_l1;
        while l <= r {
            let m = (l + r) / 2;
            let l1_bit_count = (m % L1_IN_L0_COUNT) * L1_BIT_SIZE;
            if (l1_bit_count - self.l12_indices[m].l1() as usize) < rank {
                l = m + 1;
            } else if (l1_bit_count - self.l12_indices[m].l1() as usize) >= rank {
                r = m - 1;
            } else {
                break;
            }
        }
        let l1_pos = r;
        let l12_index = &self.l12_indices[l1_pos];
        // Same problem as with L0 index.
        if l1_pos % L1_IN_L0_COUNT != 0 {
            rank -= (l1_pos % L1_IN_L0_COUNT) * L1_BIT_SIZE - l12_index.l1() as usize
        }
        let mut l2_pos = 0;
        for i in 0..l12_index.len() {
            // In contrast to L0 and L1 indices L2 indices contain the number of ones in a single block
            // and are not incremental. We therefor calculate the number of zeros by subtracting
            // the number of ones from the L2 size.
            if rank > L2_BIT_SIZE - l12_index.index(i) as usize {
                rank -= L2_BIT_SIZE - l12_index.index(i) as usize;
                l2_pos += 1;
            } else {
                break;
            }
        }
        let bit_search_start = l1_pos * L1_BIT_SIZE + (l2_pos) * L2_BIT_SIZE;
        let bit_search_end = self.bit_vec.len();
        let mut bit = 0;
        for i in bit_search_start..bit_search_end {
            if !self.bit_vec[i] {
                rank -= 1;
                if rank == 0 {
                    bit = i;
                    break;
                }
            }
        }
        if rank != 0 {
            debug_assert!(false, "rank too large");
            None
        } else {
            Some(bit)
        }
    }

    pub fn select1(&self, mut rank: usize) -> Option<usize> {
        assert!(rank > 0);
        assert!(rank <= self.bit_vec.len());
        let mut l0_pos = 0;
        // Search the L0 index with a linear search from the first ot the last L0 index.
        // If the L0 index is at one point larger than the queried rank, we use the previous
        // L0 index in which the queried position must reside.
        while l0_pos + 1 < self.l0_indices.len() && (self.l0_indices[l0_pos + 1] as usize) < rank {
            l0_pos += 1;
        }
        rank -= self.l0_indices[l0_pos] as usize;

        // Now find the L1 index in the L0 block with a binary search.
        let first_l1 = l0_pos * L1_IN_L0_COUNT;
        let last_l1 = cmp::min((l0_pos + 1) * L1_IN_L0_COUNT, self.l12_indices.len() - 1);
        let mut l = first_l1;
        let mut r = last_l1;
        // Binary search for the correct L1 block.
        while l <= r {
            let m = (l + r) / 2;
            if (self.l12_indices[m].l1() as usize) < rank {
                l = m + 1;
            } else if (self.l12_indices[m].l1() as usize) >= rank {
                r = m - 1;
            } else {
                break;
            }
        }
        let l1_pos = r;
        let l12_index = &self.l12_indices[l1_pos];
        rank -= l12_index.l1() as usize;
        let mut l2_pos = 0;
        for i in 0..l12_index.len() {
            // As long as our L2 index is smaller than our rank, we subtract the number of ones
            // from our rank and look at the next L2 block.
            if rank > l12_index.index(i) as usize {
                rank -= l12_index.index(i) as usize;
                l2_pos += 1;
            } else {
                break;
            }
        }

        // We are now inside a L2 block and do a linear search for the position of the bit.
        let bit_search_start = l1_pos * L1_BIT_SIZE + (l2_pos) * L2_BIT_SIZE;
        let bit_search_end = self.bit_vec.len();
        let mut bit = 0;
        for i in bit_search_start..bit_search_end {
            if self.bit_vec[i] {
                rank -= 1;
                if rank == 0 {
                    bit = i;
                    break;
                }
            }
        }
        // If the rank is not 0 we missed some indices/bits and the algorithm is broken or the
        // queried bit with such a rank does not exist.
        if rank != 0 {
            debug_assert!(false, "rank too large");
            None
        } else {
            Some(bit)
        }
    }

    pub fn rank0(&self, idx: usize) -> usize {
        idx - self.rank1(idx)
    }

    pub fn rank1(&self, idx: usize) -> usize {
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

    pub fn space_usage(&self) -> usize {
        (self.l12_indices.len() * 64 + self.l0_indices.len() * 64) + mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::bitvec;
    use bitvec::field::BitField;
    use bitvec::order::Lsb0;

    #[test]
    fn interleaved_index() {
        let l2s = [0b1001100111_u16, 0b1010101010_u16, 0b1100110011_u16];
        let l1 = 0b10101010_10101010_10101010_10101010_u32;
        let interleaved = InterleavedIndex::new(l1, &l2s);
        assert_eq!(interleaved.l1(), l1);
        assert_eq!(interleaved.index(0), l2s[0]);
        assert_eq!(interleaved.index(1), l2s[1]);
        assert_eq!(interleaved.index(2), l2s[2]);
    }

    #[test]
    fn small_vector() {
        let small = bitvec![u64, Lsb0; 1; 1024 + 256];
        let runaway = RunawayVector::new(&small);
        assert_eq!(runaway.l0_indices.len(), 1);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(runaway.l12_indices.len(), 1);
        assert_eq!(runaway.l12_indices[0].l1(), 0);
        assert_eq!(runaway.l12_indices[0].len(), 3);
        assert_eq!(runaway.l12_indices[0].index(0), 512);
        assert_eq!(runaway.l12_indices[0].index(1), 512);
        assert_eq!(runaway.l12_indices[0].index(2), 256);
    }

    #[test]
    fn empty_vector() {
        let mut bv = bitvec![u64, Lsb0; 0; L0_BIT_SIZE];
        bv.set(0, true);
        RunawayVector::new(&bv);
    }

    #[test]
    fn full_vector() {
        let zeros = bitvec![u64, Lsb0; 0, L0_BIT_SIZE];
        let mut bv = bitvec![u64, Lsb0; 1; L0_BIT_SIZE];
        bv.set(L0_BIT_SIZE - 1, false);
        bv.extend(zeros.iter());
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.l0_indices.len(), 2);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(runaway.l0_indices[1], u32::MAX as u64);
    }

    #[test]
    fn select0_l0() {
        let mut bv = bitvec![u64, Lsb0; 1; L0_BIT_SIZE * 2 + 1];
        bv.set(0, false);
        bv.set(1 << 32, false);
        bv.set((1 << 32) * 2, false);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(runaway.l0_indices[1], u32::MAX as u64);
        assert_eq!(runaway.l0_indices[2], 2 * u32::MAX as u64);
        assert_eq!(runaway.select0(1), Some(0));
        assert_eq!(runaway.select0(2), Some(1 << 32));
        assert_eq!(runaway.select0(3), Some((1 << 32) * 2));
    }

    #[test]
    fn select0_l1() {
        let mut bv = bitvec![u64, Lsb0; 1; L1_BIT_SIZE * 4];
        bv[0..64].store(u64::MIN);
        bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MIN);
        bv[L1_BIT_SIZE * 2..L1_BIT_SIZE * 2 + 64].store(u64::MIN);
        bv[L1_BIT_SIZE * 3..L1_BIT_SIZE * 3 + 64].store(u64::MIN);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.select0(1), Some(0));
        assert_eq!(runaway.select0(65), Some(L1_BIT_SIZE));
        assert_eq!(runaway.select0(65 + 64), Some(L1_BIT_SIZE * 2));
        assert_eq!(runaway.select0(65 + 64 * 2), Some(L1_BIT_SIZE * 3));
    }

    #[test]
    fn select0_l0_and_l1() {
        let mut bv = bitvec![u64, Lsb0; 1; L0_BIT_SIZE * 2];
        bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MIN);
        bv[L0_BIT_SIZE + L1_BIT_SIZE..L0_BIT_SIZE + L1_BIT_SIZE + 64].store(u64::MIN);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.l0_indices.len(), 2);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(L0_BIT_SIZE as u64 - runaway.l0_indices[1], 64);
        assert_eq!(runaway.select0(64), Some(L1_BIT_SIZE + 63));
        assert_eq!(runaway.select0(128), Some(L0_BIT_SIZE + L1_BIT_SIZE + 63));
    }

    #[test]
    fn select1_l0() {
        let mut bv = bitvec![u64, Lsb0; 0; L0_BIT_SIZE * 3];
        bv.set(0, true);
        bv.set(1 << 32, true);
        bv.set((1 << 32) * 2, true);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(runaway.l0_indices[1], 1);
        assert_eq!(runaway.l0_indices[2], 2);
        assert_eq!(runaway.select1(1), Some(0));
        assert_eq!(runaway.select1(2), Some(1 << 32));
        assert_eq!(runaway.select1(3), Some((1 << 32) * 2));
    }

    #[test]
    fn select1_l1() {
        let mut bv = bitvec![u64, Lsb0; 0; L1_BIT_SIZE * 4];
        bv[0..64].store(u64::MAX);
        bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MAX);
        bv[L1_BIT_SIZE * 2..L1_BIT_SIZE * 2 + 64].store(u64::MAX);
        bv[L1_BIT_SIZE * 3..L1_BIT_SIZE * 3 + 64].store(u64::MAX);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.l0_indices[0], 0);
        assert_eq!(runaway.l12_indices.len(), 4);
        assert_eq!(runaway.select1(1), Some(0));
        assert_eq!(runaway.select1(65), Some(L1_BIT_SIZE));
        assert_eq!(runaway.select1(65 + 64), Some(L1_BIT_SIZE * 2));
        assert_eq!(runaway.select1(65 + 64 * 2), Some(L1_BIT_SIZE * 3));
    }

    #[test]
    fn select1_l0_and_l1() {
        let mut bv = bitvec![u64, Lsb0; 0; L0_BIT_SIZE * 2];
        bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MAX);
        bv[L0_BIT_SIZE + L1_BIT_SIZE..L0_BIT_SIZE + L1_BIT_SIZE + 64].store(u64::MAX);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.select1(64), Some(L1_BIT_SIZE + 63));
        assert_eq!(runaway.select1(128), Some(L0_BIT_SIZE + L1_BIT_SIZE + 63));
    }

    #[test]
    fn cutoff_l2() {
        const LEN: usize = L1_BIT_SIZE + L2_BIT_SIZE - (L2_BIT_SIZE / 2);
        let mut bv = bitvec![u64, Lsb0; 0; LEN];
        bv.set(LEN - 1, true);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.rank1(1), 0);
        assert_eq!(runaway.select1(1), Some(LEN - 1));
        assert_eq!(runaway.rank0(runaway.select1(1).unwrap()), LEN - 1);
    }

    #[test]
    fn cutoff_l1() {
        const LEN: usize = L1_BIT_SIZE * 2 - (L1_BIT_SIZE / 2);
        let mut bv = bitvec![u64, Lsb0; 0; LEN];
        bv.set(LEN - 1, true);
        let runaway = RunawayVector::new(&bv);
        assert_eq!(runaway.rank1(1), 0);
        assert_eq!(runaway.select1(1), Some(LEN - 1));
        assert_eq!(runaway.rank0(runaway.select1(1).unwrap()), LEN - 1);
    }

    #[test]
    fn select1_somewhere() {
        const LEN: usize = L0_BIT_SIZE + L1_BIT_SIZE * ((L0_BIT_SIZE / L1_BIT_SIZE) / (11000));
        let mut bv = bitvec![u64, Lsb0; 0; LEN];
        bv.set(0, true);
        let mut i = L0_BIT_SIZE;
        while i < LEN {
            bv[i..i + 128].store(u128::MAX);
            i += 128;
        }
        let runaway = RunawayVector::new(&bv);
        for i in 2..LEN - L0_BIT_SIZE {
            assert_eq!(runaway.select1(i), Some(i + L0_BIT_SIZE - 2));
        }
    }

    #[test]
    fn rank1() {
        let bv = bitvec![u64, Lsb0; 1; L1_BIT_SIZE * 8192];
        let runaway = RunawayVector::new(&bv);
        for i in 0..bv.len() {
            assert_eq!(runaway.rank1(i), i);
        }
    }

    #[test]
    fn rank0() {
        let bv = bitvec![u64, Lsb0; 0; L1_BIT_SIZE * 8192];
        let runaway = RunawayVector::new(&bv);
        for i in 0..bv.len() {
            assert_eq!(runaway.rank0(i), i);
        }
    }

    #[test]
    fn rank1_l0() {
        let mut bv = bitvec![u64, Lsb0; 0; L0_BIT_SIZE * 2];
        bv[L1_BIT_SIZE..L1_BIT_SIZE + 128].store(u128::MAX);
        bv[L1_BIT_SIZE * 2..L1_BIT_SIZE * 2 + 128].store(u128::MAX);
        bv[L1_BIT_SIZE * 3..L1_BIT_SIZE * 3 + 128].store(u128::MAX);
        bv[L0_BIT_SIZE + L1_BIT_SIZE * 4689..L0_BIT_SIZE + L1_BIT_SIZE * 4689 + 128]
            .store(u128::MAX);
        let runaway = RunawayVector::new(&bv);
        for i in L1_BIT_SIZE..L1_BIT_SIZE + 128 {
            assert_eq!(runaway.rank1(i), i % L1_BIT_SIZE);
        }
        for i in L1_BIT_SIZE * 2..L1_BIT_SIZE * 2 + 128 {
            assert_eq!(runaway.rank1(i), i % L1_BIT_SIZE + 128);
        }
        for i in L1_BIT_SIZE * 3..L1_BIT_SIZE * 3 + 128 {
            assert_eq!(runaway.rank1(i), i % L1_BIT_SIZE + 128 * 2);
        }
        for i in L0_BIT_SIZE + L1_BIT_SIZE * 4689..L0_BIT_SIZE + L1_BIT_SIZE * 4689 + 128 {
            assert_eq!(runaway.rank1(i), i % L1_BIT_SIZE + 128 * 3);
        }
    }
}
