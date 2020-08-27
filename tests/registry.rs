use l10nregistry::registry::L10nRegistry;
use l10nregistry::source::FileSource;
use unic_langid::LanguageIdentifier;

use std::path::Path;

#[test]
fn test_generate_sources_for_file() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let toolkit = reg.get_source("toolkit").unwrap();
    let browser = reg.get_source("browser").unwrap();

    let mut i = reg.generate_sources_for_file(&en_us, Path::new("menu.ftl"));

    assert_eq!(i.next(), Some(toolkit));
    assert_eq!(i.next(), Some(browser));
    assert_eq!(i.next(), None);

    assert!(browser
        .fetch_file_sync(&en_us, Path::new("menu.ftl"))
        .is_none());

    let mut i = reg.generate_sources_for_file(&en_us, Path::new("menu.ftl"));
    assert_eq!(i.next(), Some(toolkit));
    assert_eq!(i.next(), None);

    assert!(toolkit
        .fetch_file_sync(&en_us, Path::new("menu.ftl"))
        .is_some());

    let mut i = reg.generate_sources_for_file(&en_us, Path::new("menu.ftl"));
    assert_eq!(i.next(), Some(toolkit));
    assert_eq!(i.next(), None);
}

#[test]
fn test_generate_source_permutations() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let toolkit = reg.get_source("toolkit").unwrap();
    let browser = reg.get_source("browser").unwrap();

    let mut i =
        reg.generate_source_permutations(&en_us, &[Path::new("menu.ftl"), Path::new("brand.ftl")]);

    assert_eq!(i.next(), Some(vec![toolkit, toolkit]));
    assert_eq!(i.next(), Some(vec![toolkit, browser]));
    assert_eq!(i.next(), Some(vec![browser, toolkit]));
    assert_eq!(i.next(), Some(vec![browser, browser]));
    assert_eq!(i.next(), None);
}

#[test]
fn test_generate_bundles_for_lang_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = &[Path::new("menu.ftl"), Path::new("brand.ftl")];
    let mut i = reg.generate_bundles_for_lang_sync(&en_us, paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[test]
fn test_generate_bundles_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = &[Path::new("menu.ftl"), Path::new("brand.ftl")];
    let langs = &[&en_us];
    let mut i = reg.generate_bundles_sync(langs, paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[tokio::test]
async fn test_generate_bundles_for_lang() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = &[Path::new("menu.ftl"), Path::new("brand.ftl")];
    let mut i = Box::pin(reg.generate_bundles_for_lang(&en_us, paths));

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}

#[tokio::test]
async fn test_generate_bundles() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );
    let fs2 = FileSource::new(
        "browser".to_string(),
        vec![en_us.clone()],
        "./data/browser/{locale}/".into(),
    );

    let mut reg = L10nRegistry::new();

    reg.register_sources(vec![fs1, fs2]).unwrap();

    let paths = &[Path::new("menu.ftl"), Path::new("brand.ftl")];
    let langs = &[&en_us];
    let mut i = Box::pin(reg.generate_bundles(langs, paths));

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}
