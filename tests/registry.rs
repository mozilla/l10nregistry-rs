use l10nregistry::registry::L10nRegistry;
use unic_langid::LanguageIdentifier;

#[test]
fn test_generate_sources_for_file() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = l10nregistry::tokio::file_source(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./tests/resources/toolkit/{locale}".into(),
    );
    let fs2 = l10nregistry::tokio::file_source(
        "browser".to_string(),
        vec![en_us.clone()],
        "./tests/resources/browser/{locale}".into(),
    );

    let mut reg = L10nRegistry::default();
    reg.register_sources(vec![fs1, fs2]).unwrap();

    {
        let lock = reg.lock();

        let toolkit = lock.get_source("toolkit").unwrap();
        let browser = lock.get_source("browser").unwrap();

        let mut i = lock.generate_sources_for_file(&en_us, "toolkit/menu.ftl");

        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), Some(browser));
        assert_eq!(i.next(), None);

        assert!(browser
            .fetch_file_sync(&en_us, "toolkit/menu.ftl")
            .is_none());

        let mut i = lock.generate_sources_for_file(&en_us, "toolkit/menu.ftl");
        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), None);

        assert!(toolkit
            .fetch_file_sync(&en_us, "toolkit/menu.ftl")
            .is_some());

        let mut i = lock.generate_sources_for_file(&en_us, "toolkit/menu.ftl");
        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), None);
    }
}

#[test]
fn test_generate_bundles_for_lang_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = l10nregistry::tokio::file_source(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./tests/resources/toolkit/{locale}".into(),
    );
    let fs2 = l10nregistry::tokio::file_source(
        "browser".to_string(),
        vec![en_us.clone()],
        "./tests/resources/browser/{locale}".into(),
    );

    let mut reg = L10nRegistry::default();
    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = vec!["toolkit/menu.ftl".into(), "browser/brand.ftl".into()];
    let mut i = reg.generate_bundles_for_lang_sync(en_us.clone(), paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[test]
fn test_generate_bundles_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = l10nregistry::tokio::file_source(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./tests/resources/toolkit/{locale}".into(),
    );
    let fs2 = l10nregistry::tokio::file_source(
        "browser".to_string(),
        vec![en_us.clone()],
        "./tests/resources/browser/{locale}".into(),
    );

    let mut reg = L10nRegistry::default();
    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = vec!["toolkit/menu.ftl".into(), "browser/brand.ftl".into()];
    let lang_ids = vec![en_us];
    let mut i = reg.generate_bundles_sync(lang_ids, paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[tokio::test]
async fn test_generate_bundles_for_lang() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = l10nregistry::tokio::file_source(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./tests/resources/toolkit/{locale}".into(),
    );
    let fs2 = l10nregistry::tokio::file_source(
        "browser".to_string(),
        vec![en_us.clone()],
        "./tests/resources/browser/{locale}".into(),
    );

    let mut reg = L10nRegistry::default();
    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = vec!["toolkit/menu.ftl".into(), "browser/brand.ftl".into()];
    let mut i = reg.generate_bundles_for_lang(en_us, paths);

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}

#[tokio::test]
async fn test_generate_bundles() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = l10nregistry::tokio::file_source(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./tests/resources/toolkit/{locale}".into(),
    );
    let fs2 = l10nregistry::tokio::file_source(
        "browser".to_string(),
        vec![en_us.clone()],
        "./tests/resources/browser/{locale}".into(),
    );

    let mut reg = L10nRegistry::default();
    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = vec!["toolkit/menu.ftl".into(), "browser/brand.ftl".into()];
    let langs = vec![en_us];
    let mut i = reg.generate_bundles(langs, paths);

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}
