use bitvec::prelude as bv;

pub struct LectureRank<'a> {
    bit_vec: &'a bv::BitVec<usize, bv::Lsb0>,
    pub s: u64,
    pub s_tick: u64,
    pub super_blocks: Vec<u64>,
    pub lookup: Vec<u8>,
}

impl<'a> LectureRank<'a> {

}

