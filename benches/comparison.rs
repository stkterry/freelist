

use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};


use freelist::{
    FreeList,
    freelist2::Freelist2,
};




pub fn benchmark(c: &mut Criterion) {
    let fl1: FreeList<usize> = FreeList::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let fl2: Freelist2<usize> = Freelist2::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let mut group = c.benchmark_group("push");
    group
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(8));

    group.bench_function("V1", |b| {
        b.iter_batched_ref(
            || { let mut fl = fl1.clone(); fl.reserve(1); fl },
            |fl| { black_box(fl.push(black_box(10))) }, 
            BatchSize::SmallInput
        );
    });
    group.bench_function("V2", |b| {
        b.iter_batched_ref(
            || { let mut fl = fl2.clone(); fl.reserve(1); fl },
            |fl| { black_box(fl.push(black_box(10))) }, 
            BatchSize::SmallInput
        );
    });
    group.finish();

    let mut group = c.benchmark_group("remove");
    group
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(8));

    group.bench_function("V1", |b| {
        b.iter_batched_ref(
            || { fl1.clone() },
            |fl| { black_box(fl.remove(black_box(0))) }, 
            BatchSize::SmallInput
        );
    });
    group.bench_function("V2", |b| {
        b.iter_batched_ref(
            || { fl2.clone() },
            |fl| { black_box(fl.remove(black_box(0))) }, 
            BatchSize::SmallInput
        );
    });
    group.finish();


    let mut group = c.benchmark_group("remove then push");
    group
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(8));

    group.bench_function("V1", |b| {
        b.iter_batched_ref(
            || black_box(fl1.clone()), 
            |fl| {
                black_box(fl.remove(black_box(1)));
                black_box(fl.push(black_box(10)));
            }, 
            BatchSize::SmallInput
        );
    });
    group.bench_function("V2", |b| {
        b.iter_batched_ref(
            || black_box(fl2.clone()), 
            |fl| {
                black_box(fl.remove(black_box(1)));
                black_box(fl.push(black_box(10)));
            }, 
            BatchSize::SmallInput
        );
    });
    group.finish();

}


criterion_group!(benches, benchmark);
criterion_main!(benches);