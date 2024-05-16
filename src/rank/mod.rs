mod naive_rank;
mod lecture_rank;

pub use naive_rank::RankNaive;
pub use lecture_rank::LectureRank;

pub trait Rankable {
    fn rank_0(&self, idx: usize) -> usize {
        idx - self.rank_1(idx)
    }
    fn rank_1(&self, idx: usize) -> usize;
}
