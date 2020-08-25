use std::path::Path;

use l10nregistry::source::FileSource;
use unic_langid::LanguageIdentifier;

#[test]
fn test_fetch_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
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

    let fs1 = FileSource::new(
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
    assert!(fs1
        .fetch_file(&en_us, Path::new("menu.ftl"))
        .await
        .is_some());
}

#[tokio::test]
async fn test_fetch_sync_2_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let fs1 = FileSource::new(
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

    let fs1 = FileSource::new(
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

#[test]
fn test_fetch_has_value_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let path = Path::new("menu.ftl");
    let path_missing = Path::new("missing.ftl");

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert_eq!(fs1.has_file(&en_us, path), None);
    assert!(fs1.fetch_file_sync(&en_us, path).is_some());
    assert_eq!(fs1.has_file(&en_us, path), Some(true));

    assert_eq!(fs1.has_file(&en_us, path_missing), None);
    assert!(fs1.fetch_file_sync(&en_us, path_missing).is_none());
    assert_eq!(fs1.has_file(&en_us, path_missing), Some(false));
}

#[tokio::test]
async fn test_fetch_has_value_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let path = Path::new("menu.ftl");
    let path_missing = Path::new("missing.ftl");

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
    );

    assert_eq!(fs1.has_file(&en_us, path), None);
    assert!(fs1.fetch_file(&en_us, path).await.is_some());
    assert_eq!(fs1.has_file(&en_us, path), Some(true));

    assert_eq!(fs1.has_file(&en_us, path_missing), None);
    assert!(fs1.fetch_file(&en_us, path_missing).await.is_none());
    assert_eq!(fs1.has_file(&en_us, path_missing), Some(false));
    assert!(fs1.fetch_file_sync(&en_us, path_missing).is_none());
}
