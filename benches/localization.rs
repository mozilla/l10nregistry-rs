use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use async_trait::async_trait;
use fluent_bundle::FluentArgs;
use fluent_fallback::{L10nKey, SyncLocalization};
use fluent_testing::{get_scenarios, get_test_file};
use l10nregistry::registry::L10nRegistry;
use l10nregistry::FileFetcher;
use l10nregistry::FileSource;

pub struct TestFileFetcher;

#[async_trait]
impl FileFetcher for TestFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        get_test_file(path)
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        get_test_file(path)
    }
}

fn preferences_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("localization/scenarios");

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

        let mut reg = L10nRegistry::default();

        let sources = scenario
            .file_sources
            .iter()
            .map(|source| {
                FileSource::new(
                    source.name.clone(),
                    source.locales.iter().map(|s| s.parse().unwrap()).collect(),
                    source.path_scheme.clone(),
                    TestFileFetcher,
                )
            })
            .collect();
        reg.register_sources(sources).unwrap();

        group.bench_function(format!("{}/format_value_sync", scenario.name), |b| {
            b.iter(|| {
                let loc = SyncLocalization::with_generator(res_ids.clone(), reg.clone());
                for key in l10n_keys.iter() {
                    loc.format_value_sync(&key.0, key.1.as_ref());
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
                let loc = SyncLocalization::with_generator(res_ids.clone(), reg.clone());
                loc.format_messages_sync(&keys);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
