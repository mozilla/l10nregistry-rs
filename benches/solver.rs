use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use l10nregistry::registry::permute_iter;
use l10nregistry::solver::testing::get_scenarios;

fn sync_bench(c: &mut Criterion) {
    let scenarios = get_scenarios();

    for scenario in scenarios {
        let mut group = c.benchmark_group("solver");

        group.bench_function(&format!("serial/{}", scenario.name), move |b| {
            b.iter(|| {
                let mut iter = permute_iter(scenario.depth, scenario.width);
                while let Some(_) = iter.next() {}
            })
        });

        group.finish();
    }
}

criterion_group!(benches, sync_bench);
criterion_main!(benches);
