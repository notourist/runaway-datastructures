use bitvec::order::Lsb0;
use bitvec::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use runaway_datastructures::rank::{LectureRank, NaiveRank, Rankable};
use std::time::Duration;

const MI_B: usize = 1024 * 1024;
const SIZE: usize = 512 * MI_B;
const SIZES: [u32; 7] = [22, 20, 18, 16, 14, 12, 10];

fn heat(c: &mut Criterion) {
    let generated = gen_bv(SIZE / 512);
    let lecture_rank = LectureRank::new(&generated);
    c.bench_function("heat1", |b| b.iter(|| lecture_rank.rank_0(2usize.pow(12))));
    c.bench_function("heat2", |b| b.iter(|| lecture_rank.rank_0(2usize.pow(12))));
}

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

fn compare_impls(c: &mut Criterion) {
    let generated = gen_bv(SIZE);
    let naive_rank = NaiveRank {
        bit_vec: &generated,
    };
    let lecture_rank = LectureRank::new(&generated);
    let mut group = c.benchmark_group("compare select");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(1000);
    for size in SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("naive/{}MiB/2^{}", SIZE / MI_B, size)),
            &size,
            |b, size| b.iter(|| naive_rank.rank_0(*size as usize)),
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("lecture/{}MiB/2^{}", SIZE / MI_B, size)),
            &size,
            |b, size| b.iter(|| lecture_rank.rank_0(*size as usize)),
        );
    }
    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    heat(c);
    compare_impls(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
