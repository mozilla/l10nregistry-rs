use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::ProblemSolver;
use l10nregistry::solver::{ParallelProblemSolver, SerialProblemSolver};
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
                let mut gen = ProblemSolver::new(res_ids.clone(), langid.clone(), reg.clone());
                while let Some(_) = <ProblemSolver as SerialProblemSolver>::next(&mut gen) {}
            })
        });

        let reg = scenario.get_l10nregistry();
        let langid: LanguageIdentifier = "en-US".parse().unwrap();

        group.bench_function(&format!("parallel/{}", scenario.name), move |b| {
            b.iter(|| {
                let mut gen = ProblemSolver::new(res_ids.clone(), langid.clone(), reg.clone());
                while let Some(_) = <ProblemSolver as ParallelProblemSolver>::next(&mut gen) {}
            })
        });

        group.finish();
    }
}

criterion_group!(benches, sync_bench);
criterion_main!(benches);
