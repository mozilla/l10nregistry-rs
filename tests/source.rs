use std::path::Path;

use l10nregistry::source::FileSource;
use unic_langid::LanguageIdentifier;

#[test]
fn test_fetch_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let mut fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert!(fs1.fetch_file_sync(&en_us, Path::new("menu.ftl")).is_some());
    assert!(fs1
        .fetch_file_sync(&en_us, Path::new("missing.ftl"))
        .is_none());
}

#[tokio::test]
async fn test_fetch_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let mut fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert!(fs1
        .fetch_file(&en_us, Path::new("menu.ftl"))
        .await
        .is_some());
    assert!(fs1
        .fetch_file(&en_us, Path::new("missing.ftl"))
        .await
        .is_none());
}

#[tokio::test]
async fn test_fetch_sync_2_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let mut fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert!(fs1.fetch_file_sync(&en_us, Path::new("menu.ftl")).is_some());
    assert!(fs1
        .fetch_file(&en_us, Path::new("menu.ftl"))
        .await
        .is_some());
    assert!(fs1.fetch_file_sync(&en_us, Path::new("menu.ftl")).is_some());
}

#[tokio::test]
async fn test_fetch_async_2_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let mut fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert!(fs1
        .fetch_file(&en_us, Path::new("menu.ftl"))
        .await
        .is_some());
    assert!(fs1.fetch_file_sync(&en_us, Path::new("menu.ftl")).is_some());
}
