use bitvec::vec::BitVec;
use memmap2::Mmap;
use runaway_datastructures::query::{Query, QueryResult};
use runaway_datastructures::runaway_vector::RunawayVector;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Instant;
use std::{env, io};
use std::ops::Sub;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let path_input = Path::new(&args[1]);
    let file_input = File::open(path_input)?;
    let start = Instant::now();
    let (queries, bit_vec) = read_file(file_input)?;

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

enum ReadState {
    FirstLine,
    Chars,
    Queries,
}

const SIZE: usize = 1074000000;

fn read_file(file: File) -> io::Result<(Vec<Query>, BitVec<u64>)> {
    let mut read_state = ReadState::FirstLine;
    let mmap = unsafe { Mmap::map(&file)? };
    let size = if file.metadata().unwrap().size() as usize > SIZE {
        SIZE
    } else {
        file.metadata().unwrap().size() as usize
    };
    let mut queries: Vec<Query> = Vec::new();
    let mut bit_vec = BitVec::with_capacity(size);
    let mut tmp_query = String::new();
    let mut page_count = 0;
    let page_count_end = mmap.len() / size;

    while page_count < page_count_end {
        let page = &mmap[page_count * size..(page_count + 1) * size];
        for i in 0..page.len() {
            match read_state {
                ReadState::FirstLine => {
                    if page[i] == b'\n' {
                        read_state = ReadState::Chars;
                    }
                }
                ReadState::Chars => {
                    if page[i] == b'1' {
                        bit_vec.push(true);
                    } else if page[i] == b'0' {
                        bit_vec.push(false);
                    } else if page[i] == b'\n' {
                        read_state = ReadState::Queries;
                    }
                }
                ReadState::Queries => {
                    if page[i] == b'\n' {
                        queries.push(Query::try_from(tmp_query.as_str()).unwrap());
                        tmp_query.clear();
                    } else {
                        tmp_query.push(page[i] as char);
                    }
                }
            }
        }
        page_count += 1;
    }
    Ok((queries, bit_vec))
}
