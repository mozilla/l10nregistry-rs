use crate::errors::L10nRegistryError;
use unic_langid::LanguageIdentifier;

pub trait LocalesProvider {
    fn locales(&self) -> Vec<LanguageIdentifier>;
}

pub trait ErrorReporter {
    fn report_errors(&self, errors: Vec<L10nRegistryError>);
}
