use bitvec::vec::BitVec;
use runaway_datastructures::query::{Query, QueryResult};
use runaway_datastructures::runaway_vector::RunawayVector;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::ops::Sub;
use std::path::Path;
use std::time::Instant;
use std::{env, io};

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let input_file = File::open(Path::new(&args[1]))?;
    let mut queries = Vec::new();
    let mut bit_vec = BitVec::new();

    let start = Instant::now();

    let mut reader = BufReader::new(input_file);
    let mut line = String::new();
    let mut line_count = 0;
    while reader.read_line(&mut line)? != 0 {
        if line_count == 1 {
            println!("{line}");
            line.chars()
                .filter(|c| *c != '\n')
                .map(|char| match char {
                    '1' => true,
                    '0' => false,
                    _ => unreachable!(),
                })
                .for_each(|bool| bit_vec.push(bool));
        } else if line_count > 1 {
            queries.push(Query::try_from(line.as_str()).unwrap());
        }
        line_count += 1;
        line.clear();
    }
    let read_elapsed = start.elapsed();

    let vector = RunawayVector::new(&bit_vec);
    let build_elapsed = start.elapsed();

    let results: Vec<QueryResult> = queries.iter().map(|query| vector.process(query)).collect();
    let build_and_process_elapsed = start.elapsed();

    let space = bit_vec.len() + vector.space_usage();
    println!(
        "RESULT name=Nasarek time={:?} build={:?} read={:?} space={} overhead={}",
        build_and_process_elapsed.sub(read_elapsed).as_millis(),
        build_elapsed.sub(read_elapsed).as_millis(),
        read_elapsed.as_millis(),
        space,
        space as f64 / bit_vec.len() as f64,
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
