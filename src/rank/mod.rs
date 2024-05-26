mod block_rank;
mod naive_rank;

pub use block_rank::BlockRank;
pub use naive_rank::NaiveRank;

pub trait Rankable {
    fn rank0(&self, idx: usize) -> usize {
        idx - self.rank1(idx)
    }
    fn rank1(&self, idx: usize) -> usize {
        idx - self.rank0(idx)
    }
}
