use l10nregistry::testing::{FileSource, RegistrySetup, TestFileFetcher};
use unic_langid::LanguageIdentifier;

const FTL_RESOURCE_TOOLKIT: &str = "toolkit/global/textActions.ftl";
const FTL_RESOURCE_BROWSER: &str = "branding/brand.ftl";

#[test]
fn test_generate_sources_for_file() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let setup = RegistrySetup::new(
        "test",
        vec![
            FileSource::new("browser", vec![en_us.clone()], "browser/{locale}"),
            FileSource::new("toolkit", vec![en_us.clone()], "toolkit/{locale}"),
        ],
        vec![en_us.clone()],
    );
    let fetcher = TestFileFetcher::new();
    let (_, reg) = fetcher.get_registry_and_environment(setup);

    {
        let lock = reg.lock();

        let toolkit = lock.get_source("toolkit").unwrap();
        let browser = lock.get_source("browser").unwrap();

        let mut i = lock.generate_sources_for_file(&en_us, FTL_RESOURCE_TOOLKIT);

        assert_eq!(i.next(), Some(browser));
        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), None);

        assert!(browser
            .fetch_file_sync(&en_us, FTL_RESOURCE_TOOLKIT, false)
            .is_none());

        let mut i = lock.generate_sources_for_file(&en_us, FTL_RESOURCE_TOOLKIT);
        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), None);

        assert!(toolkit
            .fetch_file_sync(&en_us, FTL_RESOURCE_TOOLKIT, false)
            .is_some());

        let mut i = lock.generate_sources_for_file(&en_us, FTL_RESOURCE_TOOLKIT);
        assert_eq!(i.next(), Some(toolkit));
        assert_eq!(i.next(), None);
    }
}

#[test]
fn test_generate_bundles_for_lang_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let setup = RegistrySetup::new(
        "test",
        vec![
            FileSource::new("toolkit", vec![en_us.clone()], "toolkit/{locale}"),
            FileSource::new("browser", vec![en_us.clone()], "browser/{locale}"),
        ],
        vec![en_us.clone()],
    );
    let fetcher = TestFileFetcher::new();
    let (_, reg) = fetcher.get_registry_and_environment(setup);

    let paths = vec![FTL_RESOURCE_TOOLKIT.into(), FTL_RESOURCE_BROWSER.into()];
    let mut i = reg.generate_bundles_for_lang_sync(en_us.clone(), paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[test]
fn test_generate_bundles_sync() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let setup = RegistrySetup::new(
        "test",
        vec![
            FileSource::new("toolkit", vec![en_us.clone()], "toolkit/{locale}"),
            FileSource::new("browser", vec![en_us.clone()], "browser/{locale}"),
        ],
        vec![en_us.clone()],
    );
    let fetcher = TestFileFetcher::new();
    let (_, reg) = fetcher.get_registry_and_environment(setup);

    let paths = vec![FTL_RESOURCE_TOOLKIT.into(), FTL_RESOURCE_BROWSER.into()];
    let lang_ids = vec![en_us];
    let mut i = reg.generate_bundles_sync(lang_ids, paths);

    assert!(i.next().is_some());
    assert!(i.next().is_none());
}

#[tokio::test]
async fn test_generate_bundles_for_lang() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let setup = RegistrySetup::new(
        "test",
        vec![
            FileSource::new("toolkit", vec![en_us.clone()], "toolkit/{locale}"),
            FileSource::new("browser", vec![en_us.clone()], "browser/{locale}"),
        ],
        vec![en_us.clone()],
    );
    let fetcher = TestFileFetcher::new();
    let (_, reg) = fetcher.get_registry_and_environment(setup);

    let paths = vec![FTL_RESOURCE_TOOLKIT.into(), FTL_RESOURCE_BROWSER.into()];
    let mut i = reg.generate_bundles_for_lang(en_us, paths);

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}

#[tokio::test]
async fn test_generate_bundles() {
    use futures::stream::StreamExt;

    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let setup = RegistrySetup::new(
        "test",
        vec![
            FileSource::new("toolkit", vec![en_us.clone()], "toolkit/{locale}"),
            FileSource::new("browser", vec![en_us.clone()], "browser/{locale}"),
        ],
        vec![en_us.clone()],
    );
    let fetcher = TestFileFetcher::new();
    let (_, reg) = fetcher.get_registry_and_environment(setup);

    let paths = vec![FTL_RESOURCE_TOOLKIT.into(), FTL_RESOURCE_BROWSER.into()];
    let langs = vec![en_us];
    let mut i = reg.generate_bundles(langs, paths);

    assert!(i.next().await.is_some());
    assert!(i.next().await.is_none());
}
