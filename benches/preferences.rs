use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use async_trait::async_trait;
use fluent_testing::{get_scenarios, get_test_file};
use l10nregistry::FileFetcher;
use l10nregistry::FileSource;

use l10nregistry::registry::L10nRegistry;
use unic_langid::LanguageIdentifier;

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
    let mut group = c.benchmark_group("registry/scenarios");

    for scenario in get_scenarios() {
        let res_ids = scenario.res_ids.clone();

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
                FileSource::new(
                    source.name.clone(),
                    source.locales.iter().map(|s| s.parse().unwrap()).collect(),
                    source.path_scheme.clone(),
                    TestFileFetcher,
                )
            })
            .collect();
        reg.register_sources(sources).unwrap();

        group.bench_function(format!("{}/first_bundle", scenario.name), |b| {
            b.iter(|| {
                let mut bundles = reg.generate_bundles_sync(locales.clone(), res_ids.clone());
                assert!(bundles.next().is_some());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
