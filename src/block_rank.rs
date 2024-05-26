use crate::rank::Rankable;
use bitvec::prelude as bv;
use crate::select::Selectable;

#[derive(Debug)]
pub struct BlockVector<'a> {
    bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
    pub n: usize,
    pub blocks: Vec<u64>,
}

const BLOCK_SIZE: usize = 2048;

impl<'a> BlockVector<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> Self {
        let n = bit_vec.len();

        let block_count = n / BLOCK_SIZE;
        let mut blocks: Vec<u64> = Vec::with_capacity(block_count);

        let mut incremental_ones: u64 = 0;
        for chunk in bit_vec.chunks_exact(BLOCK_SIZE) {
            incremental_ones += chunk.count_ones() as u64;
            blocks.push(incremental_ones);
        }
        BlockVector { bit_vec, n, blocks }
    }

    pub fn bit_size(&self) -> usize {
        (8 * self.blocks.len()) * 8
    }
}

impl<'a> Rankable for BlockVector<'a> {
    fn rank1(&self, idx: usize) -> usize {
        let block_idx = idx / BLOCK_SIZE;
        let bv = if block_idx != 0 {
            self.blocks[block_idx - 1] as usize
        } else {
            0
        };
        let bit_idx = idx % BLOCK_SIZE;
        let v = if bit_idx != 0 {
            self.bit_vec[(block_idx * BLOCK_SIZE)..block_idx * BLOCK_SIZE + bit_idx]
                .count_ones()
        } else {
            0
        };

        bv + v
    }
}

impl<'a> Selectable for BlockVector<'a> {
    fn select0(&self, nth: usize) -> Option<usize> {
        todo!()
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        let mut i = self.blocks.len() - 1;
        let mut rank = nth;
        while i >= 0 {
            if self.blocks[i] as usize <= rank {
                rank -= self.blocks[i] as usize;
                for bit_idx in (i + 1) * BLOCK_SIZE..(i + 2) * BLOCK_SIZE {
                    if self.bit_vec[bit_idx] {
                        rank -= 1;
                        if rank == 0 {
                            return Some((i + 1) * BLOCK_SIZE + bit_idx);
                        }
                    }
                }
            }
        }
        None
    }
}
