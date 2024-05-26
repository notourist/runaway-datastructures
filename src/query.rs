use crate::access::Accessible;
use crate::rank::Rankable;
use crate::select::Selectable;
use Query::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Query {
    Access(usize),
    Rank(bool, usize),
    Select(bool, usize),
}

impl TryFrom<&str> for Query {
    type Error = ();

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        if line.len() < 8 {
            Err(())
        } else if line[0..7] == *"access " {
            let idx = &line[7..].trim().parse::<usize>().map_err(|_| ())?;
            Ok(Access(*idx))
        } else if line[0..5] == *"rank " && line[6..7] == *" " {
            let which_bit = line[5..6].parse::<usize>().map_err(|_| ())? == 1;
            let idx = line[7..].trim().parse::<usize>().map_err(|_| ())?;
            Ok(Rank(which_bit, idx))
        } else if line[0..7] == *"select " && line[8..9] == *" " {
            let which_bit = line[7..8].parse::<usize>().map_err(|_| ())? == 1;
            let nth = line[9..].trim().parse::<usize>().map_err(|_| ())?;
            Ok(Select(which_bit, nth))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Access(bool),
    Rank(usize),
    Select(Option<usize>),
}

impl QueryResult {
    pub fn as_line(&self) -> String {
        match self {
            QueryResult::Access(b) => format!("{}\n", *b as u8),
            QueryResult::Rank(r) => format!("{}\n", r),
            QueryResult::Select(opt) => {
                opt.map_or_else(|| "None\n".to_string(), |s| format!("{}\n", s))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Query;

    #[test]
    fn access() {
        let access1 = "access 123   ";
        let access2 = "access 1535345435345\n";
        assert_eq!(Query::try_from(access1), Ok(Access(123)));
        assert_eq!(Query::try_from(access2), Ok(Access(1535345435345)));
    }

    #[test]
    fn rank() {
        let rank1 = "rank 1 1231\n";
        let rank2 = "rank 0 8898560945  ";
        assert_eq!(Query::try_from(rank1), Ok(Rank(true, 1231)));
        assert_eq!(Query::try_from(rank2), Ok(Rank(false, 8898560945)));
    }

    #[test]
    fn select() {
        let select1 = "select 0 5645456984598654\n";
        let select2 = "select 1 1\n";
        assert_eq!(
            Query::try_from(select1),
            Ok(Select(false, 5645456984598654usize))
        );
        assert_eq!(Query::try_from(select2), Ok(Select(true, 1)));
    }

    #[test]
    fn bad() {
        let bad1 = "   \n \n ";
        let bad2 = "";
        let bad3 = "select 123";
        let bad4 = "rank 1 a\n";
        assert_eq!(Query::try_from(bad1), Err(()));
        assert_eq!(Query::try_from(bad2), Err(()));
        assert_eq!(Query::try_from(bad3), Err(()));
        assert_eq!(Query::try_from(bad4), Err(()));
    }
}
