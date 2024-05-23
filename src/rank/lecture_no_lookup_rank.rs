use crate::rank::Rankable;
use bitvec::field::BitField;
use std::mem;
use std::ops::Range;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;

#[derive(Debug)]
pub struct LectureNoLookupRank<'a> {
    bit_vec: &'a BitVec<u64, Lsb0>,
    /// Number of bits in the bit vector.
    pub n: usize,
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
    pub blocks_in_super_block_count: usize,
    pub super_blocks: BitVec<u64, Lsb0>,
    /// We must use the [SuperBlock] type here because the number of zeroes in each block
    /// is added from the beginning of each super block till the end.
    pub blocks: BitVec<u64, Lsb0>,
}

impl<'a> LectureNoLookupRank<'a> {
    pub fn new(bit_vec: &'a BitVec<u64, Lsb0>) -> Self {
        let n = bit_vec.len();
        let s = (n.ilog2() / 2) as usize;
        let s_tick = s * s;

        assert!(s <= 32);
        assert!(s_tick <= 1024);

        let block_count = n / s;
        let super_block_count = n / s_tick;
        let blocks_in_super_block_count = block_count / super_block_count;

        let mut blocks: BitVec = BitVec::with_capacity(block_count * s);
        let mut super_blocks: BitVec<u64, Lsb0> = BitVec::with_capacity(super_block_count * s_tick);

        let mut incremental_ones = 0;
        let mut block_in_super_block = 0;
        for (i, chunk) in bit_vec.chunks_exact(s).enumerate() {
            incremental_ones += chunk.load::<u64>().count_ones();
            blocks[i * s..(i + 1) * s].store::<u32>(incremental_ones);


            block_in_super_block += 1;
            if block_in_super_block == blocks_in_super_block_count {
                incremental_ones += if super_blocks.is_empty() {
                    0
                } else {
                    todo!()//super_blocks[super_blocks.len() - 1]
                };
                //super_blocks.push(incremental_ones);
                incremental_ones = 0;
                block_in_super_block = 0;
            }
        }
        /*LectureNoLookupRank {
            bit_vec,
            n,
            s,
            s_tick,
            blocks_in_super_block_count,
            super_blocks,
            blocks,
        }*/
        todo!()
    }

    pub fn bit_size(&self) -> usize {
        (mem::size_of::<usize>() * 5
            + self.super_blocks.len()
            + self.blocks.len()
            + mem::size_of::<Range<usize>>())
            * 8
    }
}

impl<'a> Rankable for LectureNoLookupRank<'a> {
    fn rank_1(&self, idx: usize) -> usize {
        let super_block_idx = idx / self.s_tick;
        let sbv = if super_block_idx != 0 {
            self.super_blocks[super_block_idx - 1] as usize
        } else {
            0
        };

        let block_idx = idx / self.s;
        let bv = if block_idx % self.blocks_in_super_block_count != 0 {
            self.blocks[block_idx - 1] as usize
        } else {
            0
        };
        let bit_idx = idx % self.s;
        let v = if bit_idx != 0 {
            self.bit_vec[(block_idx * self.s)..block_idx * self.s + bit_idx]
                .load::<u32>()
                .count_ones() as usize
        } else {
            0
        };

        sbv + bv + v
    }
}
