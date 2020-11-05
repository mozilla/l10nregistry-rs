use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::SerialProblemSolver;
use unic_langid::LanguageIdentifier;

fn sync_bench(c: &mut Criterion) {
    let scenarios = get_scenarios();

    for scenario in scenarios {
        let reg = scenario.get_l10nregistry();
        let res_ids = &scenario.res_ids;

        let mut group = c.benchmark_group("solver");

        let langid: LanguageIdentifier = "en-US".parse().unwrap();

        group.bench_function(&format!("serial/{}", scenario.name), move |b| {
            b.iter(|| {
                let mut gen =
                    SerialProblemSolver::new(res_ids.clone(), langid.clone(), reg.clone());
                while let Some(_) = gen.next() {}
            })
        });

        #[cfg(feature = "tokio")]
        {
            use l10nregistry::solver::ParallelProblemSolver;
            let reg = scenario.get_l10nregistry();
            let langid: LanguageIdentifier = "en-US".parse().unwrap();

            let rt = tokio::runtime::Runtime::new().unwrap();

            group.bench_function(&format!("parallel/{}", scenario.name), move |b| {
                b.iter(|| {
                    let mut gen =
                        ParallelProblemSolver::new(res_ids.clone(), langid.clone(), reg.clone());
                    rt.block_on(async { while let Some(_) = gen.next().await {} });
                })
            });
        }

        group.finish();
    }
}

criterion_group!(benches, sync_bench);
criterion_main!(benches);
