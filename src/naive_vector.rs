use std::{cmp, mem};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::query::{Query, QueryResult};
use crate::query::Query::{Access, Rank, Select};

pub struct NaiveVector<'a> {
    bit_vec: &'a BitVec<u64, Lsb0>,
    blocks: Vec<u64>,
    block_size: usize,
}

impl<'a> NaiveVector<'a> {
    pub fn new(bit_vec: &'a BitVec<u64, Lsb0>, block_size: usize) -> Self {
        let mut blocks = Vec::with_capacity((bit_vec.len() / block_size) + 1);
        let mut incremental = 0;
        for chunk in bit_vec.chunks(block_size) {
            blocks.push(incremental);
            incremental += chunk.count_ones() as u64;
        }
        blocks.push(incremental);

        NaiveVector {
            bit_vec,
            blocks,
            block_size,
        }
    }

    pub fn rank0(&self, idx: usize) -> usize {
        assert!(idx < self.bit_vec.len());
        idx - self.rank1(idx)
    }

    pub fn rank1(&self, idx: usize) -> usize {
        assert!(idx < self.bit_vec.len());
        let block_pos = idx / self.block_size;
        let bit_pos = idx % self.block_size;
        self.blocks[block_pos] as usize + self.bit_vec[idx - bit_pos..idx].count_ones()
    }

    pub fn select0(&self, mut rank: usize) -> Option<usize> {
        assert!(rank > 0);
        assert!(rank <= self.bit_vec.len());
        let mut l = 0;
        let mut r = self.blocks.len();
        while l <= r {
            let m = (l + r) / 2;
            let block_bit_count = m * self.block_size;
            if (block_bit_count - self.blocks[m] as usize) < rank {
                l = m + 1;
            } else if block_bit_count - self.blocks[m] as usize >= rank {
                r = m - 1;
            }
        }
        let block_pos = r;
        rank -= self.blocks[block_pos] as usize;
        let bit_search_start = block_pos * self.block_size;
        let bit_search_end = cmp::min((block_pos + 1) * self.block_size, self.bit_vec.len());
        let mut u64_pos = 0;
        while rank > 64 && u64_pos < 8 {
            let zeroes = self.bit_vec
                [bit_search_start + 64 * u64_pos..bit_search_start + 64 * (u64_pos + 1)]
                .count_zeros();
            rank -= zeroes;
            u64_pos += 1;
        }
        let mut bit = 0;
        for i in bit_search_start + 64 * u64_pos..bit_search_end {
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
        let mut l = 0;
        let mut r = self.blocks.len();
        while l <= r {
            let m = (l + r) / 2;
            if (self.blocks[m] as usize) < rank {
                l = m + 1;
            } else if self.blocks[m] as usize >= rank {
                r = m - 1;
            }
        }
        let block_pos = r;
        rank -= self.blocks[block_pos] as usize;
        let bit_search_start = block_pos * self.block_size;
        let bit_search_end = cmp::min((block_pos + 1) * self.block_size, self.bit_vec.len());
        let mut u64_pos = 0;
        while rank > 64 && u64_pos < 8 {
            let ones = self.bit_vec
                [bit_search_start + 64 * u64_pos..bit_search_start + 64 * (u64_pos + 1)]
                .count_ones();
            rank -= ones;
            u64_pos += 1;
        }
        let mut bit = 0;
        for i in bit_search_start + 64 * u64_pos..bit_search_end {
            if self.bit_vec[i] {
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

    pub fn space_usage(&self) -> usize {
        (self.blocks.len() * 64) + mem::size_of::<Self>()
    }
}
