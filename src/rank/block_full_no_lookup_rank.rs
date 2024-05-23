use crate::rank::Rankable;
use bitvec::field::BitField;
use bitvec::prelude as bv;
use std::mem;

#[derive(Debug)]
pub struct BlockStaticIncrementRank<'a> {
    bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
    pub n: usize,
    pub blocks: Vec<u64>,
}

impl<'a> BlockStaticIncrementRank<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> Self {
        let n = bit_vec.len();

        let block_count = n / 64;
        let mut blocks: Vec<u64> = Vec::with_capacity(block_count);

        let mut incremental_ones: u64 = 0;
        for chunk in bit_vec.chunks_exact(64) {
            incremental_ones += chunk.load::<u64>().count_ones() as u64;
            blocks.push(incremental_ones);
        }
        BlockStaticIncrementRank { bit_vec, n, blocks }
    }

    pub fn bit_size(&self) -> usize {
        mem::size_of::<usize>() * 2
            + mem::size_of::<Vec<u64>>()
            + mem::size_of::<u64>() * self.blocks.len()
    }
}

impl<'a> Rankable for BlockStaticIncrementRank<'a> {
    fn rank_1(&self, idx: usize) -> usize {
        let block_idx = idx / 64;
        let bv = if block_idx != 0 {
            self.blocks[block_idx - 1] as usize
        } else {
            0
        };
        let bit_idx = idx % 64;
        let v = if bit_idx != 0 {
            self.bit_vec[(block_idx * 64)..block_idx * 64 + bit_idx]
                .load::<u64>()
                .count_ones() as usize
        } else {
            0
        };

        bv + v
    }
}
