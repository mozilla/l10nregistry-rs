use unic_langid::LanguageIdentifier;

pub trait LocalesProvider {
    fn locales(&self) -> &[LanguageIdentifier];
}

pub trait ErrorReporter {
    fn report_errors<E: std::error::Error>(&self, errors: Vec<E>);
}
