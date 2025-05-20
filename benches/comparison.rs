

use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};


use freelist::FreeList;


pub fn benchmark(c: &mut Criterion) {
    let freelist: FreeList<usize> = FreeList::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    c.bench_function("push", |b| {
        b.iter_batched_ref(
            || { let mut fl = freelist.clone(); fl.reserve(1); fl },
            |fl| { black_box(fl.push(black_box(10))) }, 
            BatchSize::SmallInput
        );
    });

    c.bench_function("remove", |b| {
        b.iter_batched_ref(
            || black_box(freelist.clone()),
            |fl| { black_box(fl.remove(black_box(0))) }, 
            BatchSize::SmallInput
        );
    });

    c.bench_function("remove then push", |b| {
        b.iter_batched_ref(
            || { let mut fl = freelist.clone(); fl.remove(1); fl }, 
            |fl| {
                // black_box(fl.remove(black_box(1)));
                black_box(fl.push(black_box(10)));
            }, 
            BatchSize::SmallInput
        );
    });

}


criterion_group!{
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(8));
    targets = benchmark
}
criterion_main!(benches);