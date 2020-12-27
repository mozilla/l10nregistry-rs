use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use async_trait::async_trait;
use fluent_bundle::FluentArgs;
use fluent_bundle::FluentResource;
use fluent_fallback::generator::BundleGenerator;
use fluent_fallback::{L10nKey, Localization};
use fluent_testing::{get_scenarios, get_test_file_async, get_test_file_sync};
use l10nregistry::bundles::{GenerateBundles, GenerateBundlesSync};
use l10nregistry::source::FileFetcher;
use l10nregistry::{FileSource, L10nRegistry};
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct TestFileFetcher;

#[async_trait]
impl FileFetcher for TestFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        get_test_file_sync(path)
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        get_test_file_async(path).await
    }
}

#[derive(Clone)]
struct TestBundleGenerator<'l> {
    reg: &'l L10nRegistry,
    locales: Vec<LanguageIdentifier>,
}

impl<'l> TestBundleGenerator<'l> {
    pub fn new(reg: &'l L10nRegistry, locales: Vec<LanguageIdentifier>) -> Self {
        Self { reg, locales }
    }
}

impl<'l> BundleGenerator for TestBundleGenerator<'l> {
    type Resource = Rc<FluentResource>;
    type Iter = GenerateBundlesSync<'l>;
    type Stream = GenerateBundles<'l>;

    fn bundles_iter(&self, res_ids: Vec<String>) -> Self::Iter {
        self.reg
            .generate_bundles_sync(self.locales.clone(), res_ids, None)
    }

    fn bundles_stream(&self, res_ids: Vec<String>) -> Self::Stream {
        self.reg
            .generate_bundles(self.locales.clone(), res_ids, None)
    }
}

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
        let mut reg = L10nRegistry::new();

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
        reg.register_sources(sources);

        let test = TestBundleGenerator::new(&reg, locales.clone());

        group.bench_function(format!("{}/format_value_sync", scenario.name), |b| {
            b.iter(|| {
                let loc = Localization::with_generator(res_ids.clone(), true, test.clone());
                let mut errors = vec![];

                for key in l10n_keys.iter() {
                    loc.format_value_sync(&key.0, key.1.as_ref(), &mut errors);
                }
            })
        });

        {
            let rt = tokio::runtime::Runtime::new().unwrap();

            group.bench_function(format!("{}/format_value", scenario.name), |b| {
                b.iter(|| {
                    let loc = Localization::with_generator(res_ids.clone(), true, test.clone());
                    let mut errors = vec![];

                    rt.block_on(async {
                        for key in l10n_keys.iter() {
                            loc.format_value(&key.0, key.1.as_ref(), &mut errors).await;
                        }
                    });
                })
            });
        }

        let keys: Vec<L10nKey> = l10n_keys
            .into_iter()
            .map(|key| L10nKey {
                id: key.0.into(),
                args: key.1,
            })
            .collect();
        group.bench_function(format!("{}/format_messages_sync", scenario.name), |b| {
            b.iter(|| {
                let loc = Localization::with_generator(res_ids.clone(), true, test.clone());
                let mut errors = vec![];

                loc.format_messages_sync(&keys, &mut errors);
            })
        });

        {
            let rt = tokio::runtime::Runtime::new().unwrap();

            group.bench_function(format!("{}/format_messages", scenario.name), |b| {
                b.iter(|| {
                    let loc = Localization::with_generator(res_ids.clone(), true, test.clone());
                    let mut errors = vec![];

                    rt.block_on(async {
                        loc.format_messages(&keys, &mut errors).await;
                    });
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
