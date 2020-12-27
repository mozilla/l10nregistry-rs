use fluent_bundle::FluentBundle;
use fluent_bundle::FluentResource;
use fluent_fallback::generator::BundleGenerator;
use fluent_fallback::{L10nKey, Localization};
use l10nregistry::bundles::{GenerateBundles, GenerateBundlesSync};
use l10nregistry::testing::get_test_file_source;
use l10nregistry::L10nRegistry;
use serial_test::serial;
use std::rc::Rc;
use unic_langid::{langid, LanguageIdentifier};

static LOCALES: &[LanguageIdentifier] = &[langid!("pl"), langid!("en-US")];
static mut L10N_REGISTRY: Option<L10nRegistry> = None;

const FTL_RESOURCE: &str = "toolkit/updates/history.ftl";
const L10N_ID_PL_EN: (&str, Option<&str>) = ("history-title", Some("Historia aktualizacji"));
const L10N_ID_MISSING: (&str, Option<&str>) = ("missing-id", None);
const L10N_ID_ONLY_EN: (&str, Option<&str>) = (
    "history-intro",
    Some("The following updates have been installed"),
);

struct TestBundleGenerator<'l> {
    reg: &'l L10nRegistry,
    locales: Vec<LanguageIdentifier>,
}

impl<'l> TestBundleGenerator<'l> {
    pub fn new(locales: Vec<LanguageIdentifier>, reg: &'l L10nRegistry) -> Self {
        Self { reg, locales }
    }
}

impl<'l> BundleGenerator for TestBundleGenerator<'l> {
    type Resource = Rc<FluentResource>;
    type Iter = GenerateBundlesSync<'l>;
    type Stream = GenerateBundles<'l>;

    fn bundles_iter(&self, res_ids: Vec<String>) -> Self::Iter {
        self.reg.generate_bundles_sync(
            self.locales.clone(),
            res_ids,
            Some(|bundle: &mut FluentBundle<_>| {
                bundle
                    .add_function("PLATFORM", |_positional, _named| "linux".into())
                    .expect("Failed to add a function to the bundle.");
                bundle.set_use_isolating(false);
            }),
        )
    }

    fn bundles_stream(&self, res_ids: Vec<String>) -> Self::Stream {
        self.reg.generate_bundles(
            self.locales.clone(),
            res_ids,
            Some(|bundle| {
                bundle
                    .add_function("PLATFORM", |_positional, _named| "linux".into())
                    .expect("Failed to add a function to the bundle.");
                bundle.set_use_isolating(false);
            }),
        )
    }
}

fn get_l10n_registry() -> &'static L10nRegistry {
    let reg: &mut Option<L10nRegistry> = unsafe { &mut L10N_REGISTRY };

    reg.get_or_insert_with(|| {
        let mut reg = L10nRegistry::new();

        let toolkit_fs =
            get_test_file_source("toolkit", get_app_locales().to_vec(), "toolkit/{locale}/");
        let browser_fs =
            get_test_file_source("browser", get_app_locales().to_vec(), "browser/{locale}/");

        reg.register_sources(vec![browser_fs, toolkit_fs]);
        reg
    })
}

fn get_app_locales() -> &'static [LanguageIdentifier] {
    LOCALES
}

fn sync_localization(
    reg: &'static L10nRegistry,
    res_ids: Vec<String>,
) -> Localization<TestBundleGenerator<'static>> {
    let app_locales = get_app_locales().to_vec();
    let test = TestBundleGenerator::new(app_locales, &reg);
    Localization::with_generator(res_ids, true, test)
}

fn async_localization(
    reg: &'static L10nRegistry,
    res_ids: Vec<String>,
) -> Localization<TestBundleGenerator<'static>> {
    let app_locales = get_app_locales().to_vec();
    let test = TestBundleGenerator::new(app_locales, &reg);
    Localization::with_generator(res_ids, false, test)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_sync_test() -> Localization<TestBundleGenerator<'static>> {
        sync_localization(get_l10n_registry(), vec![FTL_RESOURCE.into()])
    }

    fn setup_async_test() -> Localization<TestBundleGenerator<'static>> {
        async_localization(get_l10n_registry(), vec![FTL_RESOURCE.into()])
    }

    #[test]
    #[serial]
    fn localization_format_value_sync() {
        let loc = setup_sync_test();
        let mut errors = vec![];

        for query in &[L10N_ID_PL_EN, L10N_ID_MISSING, L10N_ID_ONLY_EN] {
            let value = loc.format_value_sync(query.0, None, &mut errors);
            let result = query.1.unwrap_or(query.0);
            assert_eq!(value, result);
        }
    }

    #[test]
    #[serial]
    fn localization_format_values_sync() {
        let loc = setup_sync_test();

        let ids = &[L10N_ID_PL_EN, L10N_ID_MISSING, L10N_ID_ONLY_EN];
        let keys = ids
            .iter()
            .map(|query| L10nKey {
                id: query.0.into(),
                args: None,
            })
            .collect::<Vec<_>>();

        let mut errors = vec![];
        let values = loc.format_values_sync(&keys, &mut errors);

        assert_eq!(values.len(), ids.len());

        for (value, query) in values.iter().zip(ids) {
            let result = query.1.unwrap_or(query.0);
            assert_eq!(value.clone(), result);
        }
    }

    // #[tokio::test]
    // #[serial]
    // async fn localization_format_value_async() {
    //     let loc = setup_async_test();
    //     let mut errors = vec![];

    //     for query in &[L10N_ID_PL_EN, L10N_ID_MISSING, L10N_ID_ONLY_EN] {
    //         let value = loc.format_value(query.0, None, &mut errors).await;
    //         let result = query.1.unwrap_or(query.0);
    //         assert_eq!(value, result);
    //     }
    // }

    // #[tokio::test]
    // #[serial]
    // async fn localization_format_values_async() {
    //     let loc = setup_async_test();

    //     let ids = &[L10N_ID_PL_EN, L10N_ID_MISSING, L10N_ID_ONLY_EN];
    //     let keys = ids
    //         .iter()
    //         .map(|query| L10nKey {
    //             id: query.0.into(),
    //             args: None,
    //         })
    //         .collect::<Vec<_>>();

    //     let mut errors = vec![];
    //     let values = loc.format_values(&keys, &mut errors).await;

    //     assert_eq!(values.len(), ids.len());

    //     for (value, query) in values.iter().zip(ids) {
    //         let result = query.1.unwrap_or(query.0);
    //         assert_eq!(value.clone(), result);
    //     }
    // }

    // #[tokio::test]
    // #[serial]
    // async fn localization_upgrade() {
    //     let mut loc = setup_sync_test();
    //     let mut errors = vec![];
    //     let value = loc.format_value_sync(L10N_ID_PL_EN.0, None, &mut errors);
    //     let expected = L10N_ID_PL_EN.1.unwrap_or(L10N_ID_PL_EN.0);
    //     assert_eq!(value, expected);

    //     loc.set_async();
    //     let value = loc.format_value(L10N_ID_PL_EN.0, None, &mut errors).await;
    //     let expected = L10N_ID_PL_EN.1.unwrap_or(L10N_ID_PL_EN.0);
    //     assert_eq!(value, expected);
    // }
}
