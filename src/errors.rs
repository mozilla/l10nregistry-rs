use fluent_bundle::FluentError;
use std::error::Error;
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone, PartialEq)]
pub enum L10nRegistryError {
    FluentError {
        path: String,
        error: FluentError,
    },
    MissingResource {
        locale: LanguageIdentifier,
        res_id: String,
    },
}

impl std::fmt::Display for L10nRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResource { locale, res_id } => {
                write!(f, "Missing resource in locale {}: {}", locale, res_id)
            }
            Self::FluentError { path, error } => write!(f, "Fluent Error in {}: {}", path, error),
        }
    }
}

impl Error for L10nRegistryError {}
