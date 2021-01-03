use fluent_bundle::FluentError;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum L10nRegistryError {
    FluentError { path: String, error: FluentError },
    MissingResource(String),
}

impl std::fmt::Display for L10nRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResource(res_id) => write!(f, "Missing resource: {}", res_id),
            Self::FluentError { path, error } => write!(f, "Fluent Error in {}: {}", path, error),
        }
    }
}

impl Error for L10nRegistryError {}
