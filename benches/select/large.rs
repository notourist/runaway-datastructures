use bitvec::order::Lsb0;
use bitvec::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use runaway_datastructures::rank::{BlockStaticIncrementRank, LectureNoLookupRank, Rankable};
use std::time::Duration;

const MI_B: usize = 1024 * 1024;

fn gen_bv(bit_length: usize) -> BitVec<u64> {
    let mut rand_bv = bitvec![u64, Lsb0; 0; bit_length];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

    let mut i = 0;
    while i < bit_length / 64 {
        let num: u64 = rng.gen();
        rand_bv[i..i + 64].store(num);
        i += 64;
    }
    rand_bv
}

fn compare_select_by_index(c: &mut Criterion) {
    const SIZE: usize = 14240 * MI_B;
    let generated = gen_bv(SIZE);
    let block_rank = BlockStaticIncrementRank::new(&generated);
    let lecture_rank = LectureNoLookupRank::new(&generated);
    let mut group = c.benchmark_group("select_large");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10000);
    group.bench_function(
        BenchmarkId::from_parameter(format!("lecture/{}MiB/{}", SIZE / MI_B, SIZE / 2)),
        |b| b.iter(|| lecture_rank.rank_0(SIZE / 2)),
    );
    group.bench_function(
        BenchmarkId::from_parameter(format!("block/{}MiB/{}", SIZE / MI_B, SIZE / 2)),
        |b| b.iter(|| block_rank.rank_0(SIZE / 2)),
    );
    group.finish();
}

criterion_group!(benches, compare_select_by_index);
criterion_main!(benches);
