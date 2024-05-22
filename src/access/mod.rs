use bitvec::prelude as bv;

pub trait Accessable {
    fn access(&self, idx: usize) -> bool;
}

pub struct NaiveAccess<'a> {
    pub bit_vec: &'a bv::BitVec<u64, bv::Lsb0>
}

impl Accessable for NaiveAccess<'_> {
    fn access(&self, idx: usize) -> bool {
        self.bit_vec[idx]
    }
}