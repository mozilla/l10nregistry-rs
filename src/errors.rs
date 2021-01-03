use std::error::Error;

#[derive(Debug, Clone)]
pub enum L10nRegistryError {
    MissingResource(String),
}

impl std::fmt::Display for L10nRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResource(res_id) => write!(f, "Missing resource: {}", res_id),
        }
    }
}

impl Error for L10nRegistryError {}
