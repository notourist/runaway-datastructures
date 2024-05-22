use crate::rank::Rankable;
use bitvec::prelude as bv;
use std::mem;

#[derive(Debug)]
pub struct BlockFullNoLookupRank<'a> {
    bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
    pub n: usize,
    pub s: usize,
    pub blocks: Vec<u64>,
}

impl<'a> BlockFullNoLookupRank<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> Self {
        let n = bit_vec.len();
        let s = (n.ilog2() / 2) as usize;

        let block_count = n / s;
        let mut blocks: Vec<u64> = Vec::with_capacity(block_count);

        let mut block_zeroes: u64 = 0;
        for block_idx in 0..block_count {
            block_zeroes += bit_vec[(block_idx * s)..(block_idx + 1) * s].count_zeros() as u64;
            blocks.push(block_zeroes);
        }
        BlockFullNoLookupRank {
            bit_vec,
            n,
            s,
            blocks,
        }
    }

    pub fn bit_size(&self) -> usize {
        mem::size_of::<usize>() * 2
            + mem::size_of::<Vec<u64>>()
            + mem::size_of::<u64>() * self.blocks.len()
    }
}

impl<'a> Rankable for BlockFullNoLookupRank<'a> {
    fn rank_0(&self, idx: usize) -> usize {
        let block_idx = idx / self.s;
        let bv = if block_idx != 0 {
            self.blocks[block_idx - 1] as usize
        } else {
            0
        };
        let bit_idx = idx % self.s;
        let v = if bit_idx != 0 {
            let bits = &self.bit_vec[(block_idx * self.s)..block_idx * self.s + bit_idx];
            bits.count_zeros()
        } else {
            0
        };

        bv + v
    }
}
