use crate::rank::Rankable;
use bitvec::field::BitField;
use std::mem;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::BitView;

type SuperBlock = u32;

#[derive(Debug)]
pub struct LectureRank<'a> {
    bit_vec: &'a BitVec<u64, Lsb0>,
    /// Number of bits in the bit vector.
    pub n: usize,
    pub n_size: usize,
    /// Number of bits in a single block:
    ///
    /// `s = floor(lg(n) / 2)`
    ///
    /// ## Limit
    /// `max s = lg(max n) / 2 = lg(2^64) / 2 = 32`
    pub s: usize,
    /// Number of bits in a super block:
    ///
    /// `s' = s * s`
    ///
    /// ## Limit
    /// `max s' = max s * max s = 32 * 32 = 1024`
    pub s_tick: usize,
    pub s_tick_size: usize,
    pub super_blocks: BitVec<u64, Lsb0>,
    /// We must use the [SuperBlock] type here because the number of zeroes in each block
    /// is added from the beginning of each super block till the end.
    pub blocks: BitVec<u64, Lsb0>,
}

impl<'a> LectureRank<'a> {
    pub fn new(bit_vec: &'a BitVec<u64, Lsb0>) -> Self {
        let n = bit_vec.len();
        let s = (n.ilog2() / 2) as usize;
        let s_tick = s * s;

        let block_count = n / s;
        let super_block_count = n / s_tick;

        let mut blocks = BitVec::with_capacity(block_count * s);
        let mut super_blocks = BitVec::with_capacity(super_block_count * s_tick);

        let s_tick_size = 1 + s_tick.ilog2() as usize;
        let n_size = 1 + n.ilog2() as usize;

        let mut all_zeroes: usize = 0;
        let mut super_block_zeroes: usize = 0;
        for (i, block_chunk) in bit_vec.chunks_exact(s).enumerate() {
            if i % (s_tick / s) == 0 {
                all_zeroes += super_block_zeroes;
                super_blocks.extend(&all_zeroes.view_bits::<Lsb0>()[..n_size]);
                super_block_zeroes = 0;
            }
            super_block_zeroes += block_chunk.count_zeros();
            blocks.extend(&super_block_zeroes.view_bits::<Lsb0>()[..s_tick_size]);
        }
        LectureRank {
            bit_vec,
            n,
            n_size,
            s,
            s_tick,
            s_tick_size,
            super_blocks,
            blocks,
        }
    }

    pub fn bit_size(&self) -> usize {
        (mem::size_of::<usize>() * 3)
            * 8
            + self.blocks.len()
            + self.super_blocks.len()
    }
}

impl<'a> Rankable for LectureRank<'a> {
    fn rank_0(&self, idx: usize) -> usize {
        let super_block_idx = idx / self.s_tick;
        let block_idx = idx / self.s;
        let bit_idx = idx % self.s;

        let mut rank = self.super_blocks[super_block_idx * self.n_size..(super_block_idx + 1) * self.n_size].load::<usize>();
        rank += self.blocks[block_idx * self.s_tick_size..(block_idx + 1) * self.s_tick_size].load::<usize>();
        rank += self.bit_vec[idx - bit_idx..idx].count_zeros();
        rank
    }
}
