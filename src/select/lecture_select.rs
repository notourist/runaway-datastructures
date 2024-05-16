use bitvec::prelude as bv;

use super::Selectable;

pub struct LectureSelect<'a> {
    pub bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
    pub k: u64,
    pub b: u64,
    pub B: Vec<u64>,
}

impl<'a> LectureSelect<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> LectureSelect<'a> {
        let k = bit_vec.count_zeros();
        let b = bit_vec.len().ilog2().ilog2();
        todo!()
    }

    fn B_select_0(&self, i: usize) -> u64 {
        todo!()
    }
}

impl<'a> Selectable for LectureSelect<'a> {
    fn select_0(&self, i: usize) -> Option<usize> {
        let sum_limit = (i) / (self.b as usize) - 1;
        let mut sum = 0u64;
        for i in 0..sum_limit {
            sum += self.B[i];
        }
        todo!()
    }

    fn select_1(&self, i: usize) -> Option<usize> {
        todo!()
    }
}