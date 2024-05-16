use std::time::Duration;
use bitvec::order::Lsb0;
use bitvec::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use runaway_datastructures::select::{NaiveSelect, Selectable};

const MI_B: usize = 1024 * 1024;
const SIZE: usize = 512 * MI_B;
const SELECTS_NAIVE: [u32; 7] = [14, 15, 16, 17, 18, 19, 20];

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

fn naive(c: &mut Criterion) {
    let naive_select = NaiveSelect { bit_vec: &gen_bv(SIZE) };
    let mut group = c.benchmark_group("naive_select");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(6));
    for select in SELECTS_NAIVE {
        group.bench_with_input(BenchmarkId::from_parameter(format!("{}MiB/2^{}", SIZE / MI_B, select)), &select, |b, select| {
            b.iter(|| {
                naive_select.select_1(2usize.pow(*select));
            })
        });
    }
    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    naive(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
