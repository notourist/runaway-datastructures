use bitvec::bitvec;
use bitvec::order::Lsb0;
use runaway_datastructures::access::NaiveAccess;
use runaway_datastructures::query::{Query, QueryResult};
use runaway_datastructures::rank::LectureRank;
use runaway_datastructures::select::{NoSelect};
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::time::{Instant};
use std::{env, io};

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let file = File::open(path)?;
    let mut lines = io::BufReader::new(file).lines().map(|l| l.unwrap());
    let mut bit_vec = bitvec![u64, Lsb0; 0; 2usize.pow(20)];

    lines.next();
    let second_line = lines.next().unwrap();
    second_line
        .trim()
        .chars()
        .map(|char| match char {
            '1' => true,
            _ => false,
            //err => panic!("Unknown char! {:?}", err),
        })
        .for_each(|bool| bit_vec.push(bool));

    let queries: Vec<Query> = lines
        .map(|line| Query::try_from(line.as_str()).unwrap())
        .collect();

    let naive_access = NaiveAccess { bit_vec: &bit_vec };
    let lecture_rank = LectureRank::new(&bit_vec);
    let select = NoSelect {};

    let start = Instant::now();
    println!("start: {:?}", start.elapsed());
    let results: Vec<QueryResult> = queries
        .iter()
        .map(|query| query.do_it(&naive_access, &lecture_rank, &select))
        .collect();
    println!("{:?}", results);
    println!("took: {:?}", start.elapsed());

    Ok(())
}
