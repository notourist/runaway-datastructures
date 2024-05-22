use crate::access::Accessible;
use crate::query::{Query, QueryResult};
use crate::rank::Rankable;
use crate::select::Selectable;

pub mod access;
pub mod query;
pub mod rank;
pub mod select;

pub struct Concrete<A: Accessible, R: Rankable, S: Selectable> {
    a: A,
    r: R,
    s: S,
}

impl<A: Accessible, R: Rankable, S: Selectable> Concrete<A, R, S> {
    pub fn process_query(&self, query: &Query) -> QueryResult {
        match query {
            Query::Access(idx) => QueryResult::Access(self.a.access(*idx)),
            Query::Rank(which, idx) => match which {
                true => QueryResult::Rank(self.r.rank_1(*idx)),
                false => QueryResult::Rank(self.r.rank_0(*idx)),
            },
            Query::Select(which, nth) => match which {
                true => QueryResult::Select(self.s.select_1(*nth)),
                false => QueryResult::Select(self.s.select_0(*nth)),
            },
        }
    }
}
