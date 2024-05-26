use bitvec::prelude as bv;
use crate::rank::NaiveRank;

use super::Selectable;

pub struct NaiveSelect<'a> {
    pub bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
}

impl<'a> NaiveSelect<'a> {
    pub fn new(bit_vec: &'a bv::BitVec<u64, bv::Lsb0>) -> Self {
        NaiveSelect {
            bit_vec
        }
    }
}

impl Selectable for NaiveSelect<'_> {
    fn select0(&self, nth: usize) -> Option<usize> {
        let mut counted = 0;
        for i in 0..self.bit_vec.len() {
            if !self.bit_vec[i] {
                counted += 1;
                if nth == counted {
                    return Some(i);
                }
            }
        }
        None
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        let mut counted = nth;
        for i in 0..self.bit_vec.len() {
            if self.bit_vec[i] {
                counted = counted.saturating_sub(1);
            }
            if counted == 0 {
                return Some(i);
            }
        }
        None
    }
}
