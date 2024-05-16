use bitvec::prelude as bv;

pub trait Accessable {
    fn access(&self, idx: usize) -> bool;
}

struct NaiveAccess<'a> {
    bit_vec: &'a bv::BitVec<usize, bv::Lsb0>
}

impl Accessable for NaiveAccess<'_> {
    fn access(&self, idx: usize) -> bool {
        self.bit_vec[idx]
    }
}