use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use fluent_bundle::FluentArgs;
use fluent_fallback::{L10nKey, Localization};
use fluent_testing::get_scenarios;
use l10nregistry::registry::L10nRegistry;
use l10nregistry::testing::get_test_file_source;
use unic_langid::LanguageIdentifier;

fn preferences_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios");

    for scenario in get_scenarios() {
        let res_ids = scenario.res_ids.clone();
        let l10n_keys: Vec<(String, Option<FluentArgs>)> = scenario
            .queries
            .iter()
            .map(|q| {
                (
                    q.input.id.clone(),
                    q.input.args.as_ref().map(|args| {
                        let mut result = FluentArgs::new();
                        for arg in args.as_slice() {
                            result.add(arg.id.clone(), arg.value.clone().into());
                        }
                        result
                    }),
                )
            })
            .collect();

        let locales: Vec<LanguageIdentifier> = scenario
            .locales
            .iter()
            .map(|l| l.parse().unwrap())
            .collect();
        let mut reg = L10nRegistry::default();

        let sources = scenario
            .file_sources
            .iter()
            .map(|source| {
                get_test_file_source(
                    &source.name,
                    source.locales.iter().map(|s| s.parse().unwrap()).collect(),
                    &source.path_scheme,
                )
            })
            .collect();
        reg.register_sources(sources);

        group.bench_function(format!("{}/format_value_sync", scenario.name), |b| {
            b.iter(|| {
                let loc = Localization::with_generator(res_ids.clone(), true, reg.clone());
                let mut errors = vec![];
                for key in l10n_keys.iter() {
                    loc.format_value_sync(&key.0, key.1.as_ref(), &mut errors);
                }
            })
        });

        let keys: Vec<L10nKey> = l10n_keys
            .into_iter()
            .map(|key| L10nKey {
                id: key.0.into(),
                args: key.1,
            })
            .collect();
        group.bench_function(format!("{}/format_messages_sync", scenario.name), |b| {
            b.iter(|| {
                let loc = Localization::with_generator(res_ids.clone(), true, reg.clone());
                let mut errors = vec![];
                loc.format_messages_sync(&keys, &mut errors);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
