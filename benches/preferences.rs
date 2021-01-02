use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use async_trait::async_trait;
use fluent_testing::get_scenarios;
use l10nregistry::FileFetcher;
use l10nregistry::FileSource;

use l10nregistry::registry::L10nRegistry;
use unic_langid::LanguageIdentifier;

pub struct TestFileFetcher;

#[async_trait]
impl FileFetcher for TestFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        fluent_testing::get_test_file_sync(path)
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        fluent_testing::get_test_file_async(path).await
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

        group.bench_function(format!("{}/sync/first_bundle", scenario.name), |b| {
            b.iter(|| {
                let mut bundles = reg.generate_bundles_sync(locales.clone(), res_ids.clone());
                assert!(bundles.next().is_some());
            })
        });

        #[cfg(feature = "tokio")]
        {
            use futures::stream::StreamExt;

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

            let rt = tokio::runtime::Runtime::new().unwrap();

            group.bench_function(&format!("{}/async/first_bundle", scenario.name), move |b| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut bundles = reg.generate_bundles(locales.clone(), res_ids.clone());
                        assert!(bundles.next().await.is_some());
                    });
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
