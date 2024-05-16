use bitvec::prelude as bv;

use super::Selectable;

pub struct NaiveSelect<'a> {
    pub bit_vec: &'a bv::BitVec<u64, bv::Lsb0>
}

impl Selectable for NaiveSelect<'_> {
    fn select_0(&self, nth: usize) -> Option<usize> {
        let mut counted = 0;
        for i in 0..self.bit_vec.len() {
            if self.bit_vec[i] == false {
                counted += 1;
                if nth == counted {
                    return Some(i)
                }
            }
        }
        None
    }

    fn select_1(&self, nth: usize) -> Option<usize> {
        let mut counted = 0;
        for i in 0..self.bit_vec.len() {
            if self.bit_vec[i] == true {
                counted += 1;
            }
            if nth == counted {
                return Some(i)
            }
        }
        None
    }
}