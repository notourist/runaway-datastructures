use bitvec::prelude as bv;

pub trait Accessible {
    fn access(&self, idx: usize) -> bool;
}

pub struct DirectAccess<'a> {
    pub bit_vec: &'a bv::BitVec<u64, bv::Lsb0>,
}

impl Accessible for DirectAccess<'_> {
    fn access(&self, idx: usize) -> bool {
        self.bit_vec[idx]
    }
}
