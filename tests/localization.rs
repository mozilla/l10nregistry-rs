use fluent_fallback::{L10nKey, Localization};
use l10nregistry::registry::L10nRegistry;
use serial_test::serial;
use std::borrow::Cow;
use unic_langid::{langid, LanguageIdentifier};

static LOCALES: &[LanguageIdentifier] = &[langid!("pl"), langid!("en-US")];
static mut L10N_REGISTRY: Option<L10nRegistry> = None;

fn get_l10n_registry() -> &'static L10nRegistry {
    let reg: &mut Option<L10nRegistry> = unsafe { &mut L10N_REGISTRY };

    reg.get_or_insert_with(|| {
        let mut reg = L10nRegistry::default();

        reg.set_lang_ids(get_app_locales().to_vec());

        let browser_fs = l10nregistry::tokio::file_source(
            "browser".to_string(),
            get_app_locales().to_vec(),
            "./tests/resources/browser/{locale}".into(),
        );
        let toolkit_fs = l10nregistry::tokio::file_source(
            "toolkit".to_string(),
            get_app_locales().to_vec(),
            "./tests/resources/toolkit/{locale}".into(),
        );

        reg.register_sources(vec![browser_fs, toolkit_fs]).unwrap();
        reg
    })
}

fn get_app_locales() -> &'static [LanguageIdentifier] {
    LOCALES
}

fn sync_localization(
    reg: &'static L10nRegistry,
    res_ids: Vec<String>,
) -> Localization<L10nRegistry> {
    Localization::with_generator(res_ids, true, reg.clone())
}

fn async_localization(
    reg: &'static L10nRegistry,
    res_ids: Vec<String>,
) -> Localization<L10nRegistry> {
    Localization::with_generator(res_ids, false, reg.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_sync_test() -> Localization<L10nRegistry> {
        sync_localization(get_l10n_registry(), vec!["browser/main.ftl".into()])
    }

    fn setup_async_test() -> Localization<L10nRegistry> {
        async_localization(get_l10n_registry(), vec!["browser/main.ftl".into()])
    }

    #[test]
    #[serial]
    fn localization_format_value_sync() {
        let loc = setup_sync_test();
        let mut errors = vec![];

        let value = loc.format_value_sync("hello-world", None, &mut errors);
        assert_eq!(value, "Hello World [browser][pl]");

        let value = loc.format_value_sync("missing-message", None, &mut errors);
        assert_eq!(value, "missing-message");

        let value = loc.format_value_sync("only-english", None, &mut errors);
        assert_eq!(value, "This message is only in English [browser][en-US]");
    }

    #[test]
    #[serial]
    fn localization_format_values_sync() {
        let loc = setup_sync_test();

        let keys = vec![
            L10nKey {
                id: "hello-world".into(),
                args: None,
            },
            L10nKey {
                id: "missing-message".into(),
                args: None,
            },
            L10nKey {
                id: "only-english".into(),
                args: None,
            },
        ];
        let mut errors = vec![];
        let values = loc.format_values_sync(&keys, &mut errors);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], Cow::Borrowed("Hello World [browser][pl]"));
        assert_eq!(values[1], Cow::Borrowed("missing-message"));
        assert_eq!(
            values[2],
            Cow::Borrowed(
                "This message is only in English [browser][en-US]"
            )
        );
    }

    #[tokio::test]
    #[serial]
    async fn localization_format_value_async() {
        let loc = setup_async_test();
        let mut errors = vec![];

        let value = loc.format_value("hello-world", None, &mut errors).await;
        assert_eq!(value, "Hello World [browser][pl]");

        let value = loc.format_value("missing-message", None, &mut errors).await;
        assert_eq!(value, "missing-message");

        let value = loc.format_value("only-english", None, &mut errors).await;
        assert_eq!(value, "This message is only in English [browser][en-US]");
    }

    #[tokio::test]
    #[serial]
    async fn localization_format_values_async() {
        let loc = setup_async_test();
        let mut errors = vec![];

        let keys = vec![
            L10nKey {
                id: "hello-world".into(),
                args: None,
            },
            L10nKey {
                id: "missing-message".into(),
                args: None,
            },
            L10nKey {
                id: "only-english".into(),
                args: None,
            },
        ];
        let values = loc.format_values(&keys, &mut errors).await;
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], Cow::Borrowed("Hello World [browser][pl]"));
        assert_eq!(values[1], Cow::Borrowed("missing-message"));
        assert_eq!(
            values[2],
            Cow::Borrowed(
                "This message is only in English [browser][en-US]"
            )
        );
    }

    #[tokio::test]
    #[serial]
    async fn localization_upgrade() {
        let mut loc = setup_sync_test();
        let mut errors = vec![];
        let value = loc.format_value_sync("hello-world", None, &mut errors);
        assert_eq!(value, "Hello World [browser][pl]");

        loc.set_async();
        let value = loc.format_value("hello-world", None, &mut errors).await;
        assert_eq!(value, "Hello World [browser][pl]");
    }

    #[tokio::test]
    #[serial]
    async fn localization_prefetch() {
        let loc = setup_sync_test();
        let mut errors = vec![];
        loc.prefetch();

        let value = loc.format_value_sync("hello-world", None, &mut errors);
        assert_eq!(value, "Hello World [browser][pl]");
    }
}
