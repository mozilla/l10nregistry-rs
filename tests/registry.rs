use fluent_testing::get_scenarios;
use l10nregistry::testing::get_test_file_source;

use async_trait::async_trait;
use fluent_bundle::FluentArgs;
use l10nregistry::source::FileFetcher;
use l10nregistry::FileSource;
use l10nregistry::L10nRegistry;

#[test]
fn registry_test() {
    for scenario in get_scenarios() {
        let mut reg = L10nRegistry::new();

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

        let mut bundles = reg.generate_bundles_sync(
            scenario
                .locales
                .iter()
                .map(|l| l.parse().unwrap())
                .collect(),
            scenario.res_ids.clone(),
            Some(|bundle| {
                bundle
                    .add_function("PLATFORM", |_positional, _named| "linux".into())
                    .expect("Failed to add a function to the bundle.");
                bundle.set_use_isolating(false);
            }),
        );

        let bundle = bundles.next().unwrap();

        let mut errors = vec![];

        for query in scenario.queries.iter() {
            let args = query.input.args.as_ref().map(|args| {
                let mut result = FluentArgs::new();
                for arg in args.as_slice() {
                    result.add(arg.id.clone(), arg.value.clone().into());
                }
                result
            });
            let msg = bundle.get_message(&query.input.id).unwrap();
            if let Some(output) = &query.output {
                if let Some(value) = &output.value {
                    let v = bundle.format_pattern(msg.value.unwrap(), args.as_ref(), &mut errors);
                    assert_eq!(v, value.as_str(), "{}", query.input.id);
                }
            }
        }
        assert_eq!(errors, vec![]);
    }
}
