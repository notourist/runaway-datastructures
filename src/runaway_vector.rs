use crate::access::Accessible;
use crate::query::Query::{Access, Rank, Select};
use crate::query::{Query, QueryResult};
use crate::rank::Rankable;
use crate::select::Selectable;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::slice::BitSliceIndex;

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

        let mut l1: u32 = 0;
        let mut l2s: [u16; 4] = [0; 4];
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
        if bit_vec.len() % L0_BIT_SIZE != 0 {
            l0_indices.push(l1 as u64 + l0_indices.last().unwrap_or(&0_u64));
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
        return None;
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        assert!(nth > 0);
        assert!(nth <= self.bit_vec.len());
        let mut rank = nth;
        let mut l0_index = 0;
        while l0_index < self.l0_indices.len() {
            if self.l0_indices[l0_index] as usize >= nth {
                l0_index -= 1;
                break;
            } else if l0_index + 1 == self.l0_indices.len() {
                break;
            }
            l0_index += 1;
        }
        rank -= self.l0_indices[l0_index] as usize;
        if rank == 0 {
            return Some(l0_index * L0_BIT_SIZE);
        }

        let first_l1 = l0_index * L0_BIT_SIZE / L1_BIT_SIZE;
        let last_l1 =
            if (l0_index + 1) * (L0_BIT_SIZE / L1_BIT_SIZE) > self.l12_indices.len() - 1 {
                self.l12_indices.len() - 1
            } else {
                (l0_index + 1) * (L0_BIT_SIZE / L1_BIT_SIZE)
            };

        let mut l = first_l1;
        let mut r = last_l1;
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
        let l1_index = r;
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
            return Some(l1_index * L1_BIT_SIZE + l2_index * L2_BIT_SIZE);
        }
        let bit_search_start = l1_index * L1_BIT_SIZE + (l2_index) * L2_BIT_SIZE;
        let bit_search_end = if bit_search_start + L2_BIT_SIZE > self.bit_vec.len() {
            self.bit_vec.len()
        } else {
            bit_search_start + L2_BIT_SIZE
        };
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
        if rank != 0 {
            panic!("rank too large")
        } else {
            Some(l1_index * L1_BIT_SIZE + l2_index * L2_BIT_SIZE + (bit % L2_BIT_SIZE))
        }
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

    #[test]
    fn select1_l0() {
        let mut rand_bv = bitvec![u64, Lsb0; 0; L0_BIT_SIZE * 4];
        rand_bv.set(0, true);
        rand_bv.set(1 << 32, true);
        rand_bv.set((1 << 32) * 2, true);
        let runaway = RunawayVector::new(&rand_bv);
        assert_eq!(runaway.select1(1), Some(0));
        assert_eq!(runaway.select1(2), Some(1 << 32));
        assert_eq!(runaway.select1(3), Some((1 << 32) * 2));
    }

    #[test]
    fn select1_l1() {
        const BIT_VEC_LEN: usize = L1_BIT_SIZE * 4;
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        rand_bv[0..64].store(u64::MAX);
        rand_bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MAX);
        rand_bv[L1_BIT_SIZE * 2..L1_BIT_SIZE * 2 + 64].store(u64::MAX);
        rand_bv[L1_BIT_SIZE * 3..L1_BIT_SIZE * 3 + 64].store(u64::MAX);
        let runaway = RunawayVector::new(&rand_bv);
        assert_eq!(runaway.select1(1), Some(0));
        assert_eq!(runaway.select1(65), Some(L1_BIT_SIZE));
        assert_eq!(runaway.select1(65 + 64), Some(L1_BIT_SIZE * 2));
        assert_eq!(runaway.select1(65 + 64 * 2), Some(L1_BIT_SIZE * 3));
    }

    #[test]
    fn select1_l0_and_l1() {
        const BIT_VEC_LEN: usize = L0_BIT_SIZE * 2;
        let mut rand_bv = bitvec![u64, Lsb0; 0; BIT_VEC_LEN];
        rand_bv[L1_BIT_SIZE..L1_BIT_SIZE + 64].store(u64::MAX);
        rand_bv[L0_BIT_SIZE + L1_BIT_SIZE..L0_BIT_SIZE + L1_BIT_SIZE + 64].store(u64::MAX);
        let runaway = RunawayVector::new(&rand_bv);
        assert_eq!(runaway.select1(64), Some(L1_BIT_SIZE + 63));
        assert_eq!(runaway.select1(128), Some(L0_BIT_SIZE + L1_BIT_SIZE + 63));
    }
}
