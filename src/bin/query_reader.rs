use bitvec::bitvec;
use bitvec::order::Lsb0;
use runaway_datastructures::query::{Query, QueryResult};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;
use std::time::Instant;
use std::{env, io};
use runaway_datastructures::runaway_vector::RunawayVector;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let path_input = Path::new(&args[1]);
    let file_input = File::open(path_input)?;
    let mut lines = io::BufReader::new(file_input).lines().map(|l| l.unwrap());
    let mut bit_vec = bitvec![u64, Lsb0;];

    lines.next();
    let second_line = lines.next().unwrap();
    second_line
        .trim()
        .chars()
        .map(|char| match char {
            '1' => true,
            '0' => false,
            err => panic!("Unknown char! {:?}", err),
        })
        .for_each(|bool| bit_vec.push(bool));

    let queries: Vec<Query> = lines
        .map(|line| Query::try_from(line.as_str()).unwrap())
        .collect();

    let start = Instant::now();
    let vector = RunawayVector::new(&bit_vec);
    let results: Vec<QueryResult> = queries
        .iter()
        .map(|query| vector.process(query))
        .collect();

    let time = start.elapsed();
    let space = bit_vec.len() + vector.bit_size();
    println!(
        "RESULT name=Nasarek time={:?} space={} overhead={}",
        time,
        space,
        vector.bit_size() as f64 / space as f64,
    );

    let path_output = Path::new(&args[2]);
    let mut file_output = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path_output)?;
    results
        .iter()
        .map(|result| result.as_line())
        .for_each(|line| {
            file_output.write_all(line.as_bytes()).unwrap();
        });

    Ok(())
}
