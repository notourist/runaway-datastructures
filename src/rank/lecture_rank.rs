use std::mem;
use std::ops::Range;
use crate::rank::Rankable;
use bitvec::field::BitField;
use bitvec::prelude as bv;

type SuperBlock = u32;

#[derive(Debug)]
pub struct LectureRank<'a> {
    bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
    /// Number of bits in the bit vector.
    pub n: usize,
    /// Number of bits in a single block:
    ///
    /// `s = floor(lg(n) / 2)`
    ///
    /// ## Limit
    /// `max s = lg(max n) / 2 = lg(2^64) / 2 = 32`
    ///
    /// ## Type
    /// `lg(max s) = lg(32) = 5 => u8`
    pub s: usize,
    /// Number of bits in a super block:
    ///
    /// `s' = s * s`
    ///
    /// ## Limit
    /// `max s' = max s * max s = 32 * 32 = 1024`
    ///
    /// ## Type
    /// `lg(max s') = lg(1024) = 10 => u16`
    pub s_tick: usize,
    pub blocks_in_super_block_count: usize,
    pub super_blocks: Vec<SuperBlock>,
    /// We must use the [SuperBlock] type here because the number of zeroes in each block
    /// is added from the beginning of each super block till the end.
    pub blocks: Vec<SuperBlock>,
    pub lookup: Vec<Vec<u8>>,
    pub unaccounted_count: usize,
    pub unaccounted_range: Range<usize>,
}

impl<'a> LectureRank<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> Self {
        let n = bit_vec.len();
        let s = (n.ilog2() / 2) as usize;
        let s_tick = s * s;

        assert!(s <= 32);
        assert!(s_tick <= 1024);

        let max_vector = 2u32.pow(s as u32) - 1;
        let mut lookup: Vec<Vec<u8>> = Vec::with_capacity(max_vector as usize);

        // For every 's' length bit vector...
        for vector in 0..=max_vector {
            lookup.push(vec![0; s]);
            // For every position i...
            let mut zero_count: u8 = 0;
            for i in 0..s {
                lookup[vector as usize][i] = zero_count;
                if (vector >> i & 1) == 0 {
                    zero_count += 1;
                }
            }
        }

        let block_count = n / s;
        let super_block_count = n / s_tick;
        let blocks_in_super_block_count = block_count / super_block_count;

        let mut blocks: Vec<SuperBlock> = Vec::with_capacity(block_count);
        let mut super_blocks: Vec<SuperBlock> = Vec::with_capacity(super_block_count);

        let mut block_zero_count: SuperBlock = 0;
        let mut current_block_in_sb_count = 0;
        for block_idx in 0..block_count {
            let bits_in_block = &bit_vec[(block_idx * s)..(block_idx + 1) * s];
            block_zero_count += bits_in_block.count_zeros() as SuperBlock;
            blocks.push(block_zero_count);
            current_block_in_sb_count += 1;
            if current_block_in_sb_count == blocks_in_super_block_count {
                block_zero_count += if super_blocks.is_empty() {
                    0
                } else {
                    super_blocks[super_blocks.len() - 1]
                };
                super_blocks.push(block_zero_count);
                block_zero_count = 0;
                current_block_in_sb_count = 0;
            }
        }
        // Dirty hack: the last block is too small and was ignored during construction,
        // so we need to lookup bits by hand
        let unaccounted_count = n % s;
        let unaccounted_range = n - unaccounted_count..n;
        LectureRank {
            bit_vec,
            n,
            s,
            s_tick,
            blocks_in_super_block_count,
            super_blocks,
            blocks,
            lookup,
            unaccounted_count,
            unaccounted_range,
        }
    }

    pub fn bit_size(&self) -> usize {
        (mem::size_of::<usize>()
        + mem::size_of::<usize>()
        + mem::size_of::<usize>()
        + mem::size_of::<usize>()
        + mem::size_of::<SuperBlock>() * self.super_blocks.len()
        + mem::size_of::<SuperBlock>() * self.blocks.len()
        + mem::size_of::<u8>() * self.lookup.len() + self.lookup[0].len()
        + mem::size_of::<usize>()
        + mem::size_of::<Range<usize>>()) * 8
    }
}

impl<'a> Rankable for LectureRank<'a> {
    fn rank_0(&self, idx: usize) -> usize {
        let super_block_idx = idx / self.s_tick;
        let sbv = if super_block_idx != 0 {
            self.super_blocks[super_block_idx - 1] as usize
        } else {
            0
        };

        let block_idx = idx / self.s ;
        let bv = if block_idx % self.blocks_in_super_block_count != 0 {
            self.blocks[block_idx - 1] as usize
        } else {
            0
        };
        let bit_idx = idx % self.s ;
        let v = if bit_idx != 0 {
            let num = if self.unaccounted_range.contains(&idx) && self.unaccounted_count != 0 {
                self.bit_vec[(self.n - self.unaccounted_count)..self.n].load::<u32>()
            } else {
                self.bit_vec[(block_idx * self.s )..(block_idx + 1) * self.s ].load::<u32>()
            };
            let zero_counts = &self.lookup[num as usize];
            zero_counts[bit_idx] as usize
        } else {
            0
        };

        sbv + bv + v
    }
}
