use bitvec::order::Lsb0;
use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use runaway_datastructures::rank::{BlockRank, InterleavedRank, Rankable};
use std::time::Duration;

const INDICES: [u32; 7] = [22, 20, 18, 16, 14, 12, 10];

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
    const MI_B: usize = 1024 * 1024;

    const SIZE: usize = 512 * MI_B;
    let generated = gen_bv(SIZE);
    let block_rank = BlockRank::new(&generated);
    let interleaved_rank = InterleavedRank::new(&generated);
    let mut group = c.benchmark_group("rank_index");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(1000);
    for idx in INDICES {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("interleaved/{}MiB/2^{}", SIZE / MI_B, idx)),
            &idx,
            |b, size| b.iter(|| black_box(interleaved_rank.rank0(*size as usize))),
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("block/{}MiB/2^{}", SIZE / MI_B, idx)),
            &idx,
            |b, size| b.iter(|| black_box(block_rank.rank0(*size as usize))),
        );
    }
    group.finish();
}

criterion_group!(benches, compare_select_by_index);
criterion_main!(benches);