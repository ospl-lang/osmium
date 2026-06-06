use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

pub fn loop_to(c: &mut Criterion) {
    c.bench_function("loop to 100", |b| {
        b.iter_batched(
            || (ospl_vm::VM::new(), ospl_vm::tests::cond::loops_code(100)),
            |(mut vm, code)| {
                vm.run_all(&code);
            },
            criterion::BatchSize::LargeInput
        );
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(10000, Output::Flamegraph(None)));

    targets = loop_to
}
criterion_main!(benches);
