use crate::rank::Rankable;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;

const L0_BIT_SIZE: usize = 1 << 32;
const L1_BIT_SIZE: usize = 2048;
const L2_BIT_SIZE: usize = 512;

const L1_INDEX_BIT_SIZE: usize = 32;
const L2_INDEX_BIT_SIZE: usize = 10;

#[derive(Debug)]
struct InterleavedIndex(u64);

impl InterleavedIndex {
    pub fn new(l1: u32, l2s: &[u16]) -> Self {
        assert_eq!(l2s.len(), 3);
        let mut value = l1 as u64;

        for (i, l2) in l2s.iter().enumerate() {
            value |= (*l2 as u64) << L2_INDEX_BIT_SIZE * i + L1_INDEX_BIT_SIZE
        }

        InterleavedIndex(value)
    }

    fn l1(&self) -> u32 {
        self.0 as u32
    }

    fn index(&self, index: usize) -> u16 {
        assert!(index < 4);
        (self.0 >> ((L2_INDEX_BIT_SIZE * index) + L1_INDEX_BIT_SIZE) & (1 << L2_INDEX_BIT_SIZE) - 1)
            as u16
    }
}

#[derive(Debug)]
pub struct InterleavedRank<'a> {
    bit_vec: &'a BitVec<u64, Lsb0>,
    l12_indices: Vec<InterleavedIndex>,
    l0_indices: Vec<u64>,
}

impl<'a> InterleavedRank<'a> {
    pub fn new(bit_vec: &'a BitVec<u64, Lsb0>) -> Self {
        let l0_capacity = (bit_vec.len() / L0_BIT_SIZE) + 1;
        let l12_capacity = (bit_vec.len() / 64) + 1;

        let mut l0_indices: Vec<u64> = Vec::with_capacity(l0_capacity);
        let mut l12_indices: Vec<InterleavedIndex> = Vec::with_capacity(l12_capacity);

        let mut l0: u64 = 0;
        let mut l1: u32 = 0;
        let mut l2s: [u16; 4] = [0; 4];
        for (i, chunk) in bit_vec.chunks(L2_BIT_SIZE).enumerate() {
            if i % (L0_BIT_SIZE / L2_BIT_SIZE) == 0 {
                l0_indices.push(l0);
                l1 = 0;
            }
            let l2_pos = i % 4;
            let l2 = chunk.count_ones() as u16;
            l2s[l2_pos] = l2;
            if i % (L1_BIT_SIZE / L2_BIT_SIZE) == 3 {
                l12_indices.push(InterleavedIndex::new(l1, &l2s[0..3]));
                l1 = l1 + l2s[0] as u32 + l2s[1] as u32 + l2s[2] as u32 + l2s[3] as u32;
                l0 += l1 as u64;
                l2s = [0; 4];
            }
        }
        // Small vector fix
        if l12_indices.len() == 0 {
            l12_indices.push(InterleavedIndex::new(l1, &l2s[0..3]));
        }
        InterleavedRank {
            bit_vec,
            l0_indices,
            l12_indices,
        }
    }

    pub fn bit_size(&self) -> usize {
        self.l12_indices.len() * 64 + self.l0_indices.len() * 64
    }
}

impl<'a> Rankable for InterleavedRank<'a> {
    fn rank_1(&self, idx: usize) -> usize {
        assert!(idx < self.bit_vec.len());
        let l0_pos = idx / L0_BIT_SIZE;
        let l1_pos = idx / L1_BIT_SIZE;
        let l2_pos = (idx / L2_BIT_SIZE) % 4;
        let bit_idx = idx % L2_BIT_SIZE;

        //dbg!(l0_pos);
        //dbg!(l1_pos);
        //dbg!(l2_pos);
        //dbg!(bit_idx);

        let l0: usize = self.l0_indices[l0_pos] as usize;
        //dbg!(l0);
        let l12 = &self.l12_indices[l1_pos];
        let l1: usize = l12.l1() as usize;
        //dbg!(l1);
        let mut l2: usize = 0;
        for i in 0..l2_pos {
            //dbg!(l12.index(i) as usize);
            l2 += l12.index(i) as usize;
        }
        //dbg!(l2);
        let hand_counted = self.bit_vec[idx - bit_idx..idx].count_ones();
        //println!("{:?}", self.bit_vec[idx - bit_idx..idx].to_string());
        //dbg!(hand_counted);
        l0 + l1 + l2 + hand_counted
    }
}
