mod naive_select;

pub use naive_select::NaiveSelect;

pub struct NoSelect {}
impl Selectable for NoSelect {
    fn select_0(&self, _: usize) -> Option<usize> {
        None
    }

    fn select_1(&self, _: usize) -> Option<usize> {
        None
    }
}
pub trait Selectable {
    fn select_0(&self, i: usize) -> Option<usize>;

    fn select_1(&self, i: usize) -> Option<usize>;
}
