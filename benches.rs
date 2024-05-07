use core::mem::align_of;

use criterion::{criterion_group, criterion_main, Criterion};

use rand::{rngs::StdRng, Rng, SeedableRng};
use type_layout::TypeLayout;

#[derive(Copy, Clone)]
#[repr(C, align(16))]
struct AlignedU128(u128);

const GAP: usize = 128;

#[derive(TypeLayout, Copy, Clone)]
#[repr(C)]
struct EnforcedAlignment {
    _offset: [u8; GAP - 8],
    data: AlignedU128,
}

#[derive(TypeLayout, Copy, Clone)]
#[repr(C)]
struct StockAlignment {
    _offset: [u8; GAP - 8],
    data: u128,
}

const N: usize = 8192;

/// This function is used to clear the cache before each benchmark
/// to avoid the effect of cache hits on the benchmark results.
fn cache_fuzzer() -> Vec<usize> {
    let mut cache_clearer = vec![];
    for i in 0..N * 1000 {
        cache_clearer.push(i);
    }
    cache_clearer
}

pub fn alignment_bench(c: &mut Criterion) {
    println!("Alignment of u128: {}", align_of::<u128>());
    println!("{}", StockAlignment::type_layout());
    println!("{}", EnforcedAlignment::type_layout());

    let mut stock_aligned = Vec::with_capacity(N);
    let mut enforced_aligned = Vec::with_capacity(N);
    for i in 0..N {
        enforced_aligned.push(EnforcedAlignment {
            _offset: [0; GAP - 8],
            data: AlignedU128(i as u128),
        });
        stock_aligned.push(StockAlignment {
            _offset: [0; GAP - 8],
            data: i as u128,
        });
    }
    let mut ordered = c.benchmark_group("ordered access");
    ordered.bench_function("stock align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in 0..N {
                    sum += stock_aligned[i].data;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });

    ordered.bench_function("enforced align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in 0..N {
                    sum += enforced_aligned[i].data.0;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });

    ordered.finish();

    let mut reversed = c.benchmark_group("reversed access");
    reversed.bench_function("stock align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in (0..N).rev() {
                    sum += stock_aligned[i].data;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });
    reversed.bench_function("enforced align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in (0..N).rev() {
                    sum += enforced_aligned[i].data.0;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });
    reversed.finish();

    let mut rng = StdRng::seed_from_u64(1);
    let rand_idxs = (0..N).map(|_| rng.gen_range(0..N)).collect::<Vec<_>>();
    let mut randomized = c.benchmark_group("randomized access");
    randomized.bench_function("stock align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in 0..N {
                    sum += stock_aligned[rand_idxs[i]].data;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });

    randomized.bench_function("enforced align", |b| {
        b.iter_batched(
            cache_fuzzer,
            |_| {
                let mut sum = 0;
                for i in 0..N {
                    sum += enforced_aligned[rand_idxs[i]].data.0;
                }
                sum
            },
            criterion::BatchSize::PerIteration,
        );
    });
    randomized.finish();
}

criterion_group!(benches, alignment_bench);
criterion_main!(benches);
