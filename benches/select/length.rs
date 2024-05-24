use bitvec::order::Lsb0;
use bitvec::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use runaway_datastructures::rank::{BlockStaticIncrementRank, InterleavedRank, NaiveRank, Rankable};
use std::time::Duration;

const SIZES: [u32; 5] = [8, 10, 12, 14, 16];

fn compare_select_by_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_length");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(1000);
    for size in SIZES {
        let generated = gen_bv(2usize.pow(size));
        let naive_rank = NaiveRank {
            bit_vec: &generated,
        };
        let block_rank = BlockStaticIncrementRank::new(&generated);
        let interleaved_rank = InterleavedRank::new(&generated);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("naive/{}MiB/2^{}", size, size)),
            &size,
            |b, size| b.iter(|| naive_rank.rank_0(*size as usize - 1)),
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("lecture/{}MiB/2^{}", size, size)),
            &size,
            |b, size| b.iter(|| interleaved_rank.rank_0(*size as usize - 1)),
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("block/{}MiB/2^{}", size, size)),
            &size,
            |b, size| b.iter(|| block_rank.rank_0(*size as usize - 1)),
        );
    }
    group.finish();
}

pub fn gen_bv(bit_length: usize) -> BitVec<u64> {
    let mut rand_bv = bitvec::bitvec![u64, Lsb0; 0; bit_length];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

    let mut i = 0;
    while i < bit_length / 64 {
        let num: u64 = rng.gen();
        rand_bv[i..i + 64].store(num);
        i += 64;
    }
    rand_bv
}

criterion_group!(benches, compare_select_by_length);
criterion_main!(benches);
