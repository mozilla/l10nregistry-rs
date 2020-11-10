use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use l10nregistry::registry::L10nRegistry;

const res_ids: &[&str] = &[
    "branding/brand.ftl",
    "browser/branding/brandings.ftl",
    "browser/branding/sync-brand.ftl",
    "browser/preferences/preferences.ftl",
    "browser/preferences/fonts.ftl",
    "toolkit/featuregates/features.ftl",
    "browser/preferences/addEngine.ftl",
    "browser/preferences/blocklists.ftl",
    "browser/preferences/clearSiteData.ftl",
    "browser/preferences/colors.ftl",
    "browser/preferences/connection.ftl",
    "browser/preferences/languages.ftl",
    "browser/preferences/permissions.ftl",
    "browser/preferences/selectBookmark.ftl",
    "browser/preferences/siteDataSettings.ftl",
    "browser/aboutDialog.ftl",
    "browser/sanitize.ftl",
    "toolkit/updates/history.ftl",
    "security/certificates/deviceManager.ftl",
    "security/certificates/certManager.ftl",
];

fn preferences_bench(c: &mut Criterion) {
    let locales = vec!["en-US".parse().unwrap()];
    c.bench_function("preferences", move |b| {
        b.iter(|| {
            let mut reg = L10nRegistry::default();

            reg.set_lang_ids(locales.clone());

            let browser_fs = l10nregistry::tokio::file_source(
                "browser".to_string(),
                locales.clone(),
                "./tests/resources/browser/{locale}".into(),
            );
            let toolkit_fs = l10nregistry::tokio::file_source(
                "toolkit".to_string(),
                locales.clone(),
                "./tests/resources/toolkit/{locale}".into(),
            );

            reg.register_sources(vec![toolkit_fs, browser_fs]).unwrap();

            let paths = res_ids.iter().map(|&r| r.into()).collect();
            let mut i = reg.generate_bundles_for_lang_sync(locales[0].clone(), paths);

            assert!(i.next().is_some());
        })
    });
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
