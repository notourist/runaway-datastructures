mod naive_select;
mod lecture_select;

pub use naive_select::NaiveSelect;
pub use lecture_select::LectureSelect;

pub trait Selectable {
    fn select_0(&self, i: usize) -> Option<usize>;

    fn select_1(&self, i: usize) -> Option<usize>;
}

