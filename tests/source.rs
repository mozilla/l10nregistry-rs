use futures::future::join_all;
use l10nregistry::source::FileSource;
use unic_langid::LanguageIdentifier;

fn fetch_sync(path: &str) -> Result<Option<String>, std::io::Error> {
    if !std::path::Path::new(path).exists() {
        return Ok(None);
    }
    Ok(Some(std::fs::read_to_string(path)?))
}

#[test]
fn test_fetch_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    assert!(fs1.fetch_file_sync(&en_us, "menu.ftl").is_some());
    assert!(fs1.fetch_file_sync(&en_us, "missing.ftl").is_none());
}

#[tokio::test]
async fn test_fetch_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    assert!(fs1.fetch_file(&en_us, "menu.ftl").await.is_some());
    assert!(fs1.fetch_file(&en_us, "missing.ftl").await.is_none());
    assert!(fs1.fetch_file(&en_us, "menu.ftl").await.is_some());
}

#[tokio::test]
async fn test_fetch_sync_2_async() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    assert!(fs1.fetch_file_sync(&en_us, "menu.ftl").is_some());
    assert!(fs1.fetch_file(&en_us, "menu.ftl").await.is_some());
    assert!(fs1.fetch_file_sync(&en_us, "menu.ftl").is_some());
}

#[tokio::test]
async fn test_fetch_async_2_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    assert!(fs1.fetch_file(&en_us, "menu.ftl").await.is_some());
    assert!(fs1.fetch_file_sync(&en_us, "menu.ftl").is_some());
}

#[test]
fn test_fetch_has_value_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let path = "menu.ftl";
    let path_missing = "missing.ftl";

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
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
    let path = "menu.ftl";
    let path_missing = "missing.ftl";

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    assert_eq!(fs1.has_file(&en_us, path), None);
    assert!(fs1.fetch_file(&en_us, path).await.is_some());
    println!("Completed");
    assert_eq!(fs1.has_file(&en_us, path), Some(true));

    assert_eq!(fs1.has_file(&en_us, path_missing), None);
    assert!(fs1.fetch_file(&en_us, path_missing).await.is_none());
    assert_eq!(fs1.has_file(&en_us, path_missing), Some(false));
    assert!(fs1.fetch_file_sync(&en_us, path_missing).is_none());
}

#[tokio::test]
async fn test_fetch_async_consequitive() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();

    let fs1 = FileSource::new(
        "toolkit".to_string(),
        vec![en_us.clone()],
        "./data/toolkit/{locale}/".into(),
        fetch_sync,
    );

    let results = join_all(vec![
        fs1.fetch_file(&en_us, "menu.ftl"),
        fs1.fetch_file(&en_us, "menu.ftl"),
    ])
    .await;
    assert!(results[0].is_some());
    assert!(results[1].is_some());

    assert!(fs1.fetch_file(&en_us, "menu.ftl").await.is_some());
}
