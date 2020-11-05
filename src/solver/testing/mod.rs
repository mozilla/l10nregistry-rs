mod scenarios;

use crate::registry::L10nRegistry;
use crate::source::FileSource;
use crate::FileFetcher;
use async_trait::async_trait;
pub use scenarios::get_scenarios;

#[derive(Clone)]
pub struct MockFileFetcher {
    files: Vec<(String, String)>,
}

impl MockFileFetcher {
    pub fn new<T: ToString>(files: Vec<(T, T)>) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(p, v)| (p.to_string(), v.to_string()))
                .collect(),
        }
    }
}

#[async_trait]
impl FileFetcher for MockFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        for (p, value) in &self.files {
            if p == path {
                return Ok(value.clone());
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"))
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        self.fetch_sync(path)
    }
}

pub struct Scenario {
    pub name: String,
    files: Vec<String>,
    sources: Vec<(String, Vec<String>, String)>,
    pub res_ids: Vec<String>,
    pub solutions: Option<Vec<Vec<usize>>>,
}

impl Scenario {
    pub fn new<S: ToString>(
        name: S,
        files: Vec<S>,
        sources: Vec<(S, Vec<S>, S)>,
        res_ids: Vec<S>,
        solutions: Option<Vec<Vec<usize>>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            files: files.iter().map(|p| p.to_string()).collect(),
            sources: sources
                .iter()
                .map(|(name, langids, pre_path)| {
                    (
                        name.to_string(),
                        langids.iter().map(|l| l.to_string()).collect(),
                        pre_path.to_string(),
                    )
                })
                .collect(),
            res_ids: res_ids.iter().map(|r| r.to_string()).collect(),
            solutions,
        }
    }

    pub fn get_l10nregistry(&self) -> L10nRegistry {
        let mut reg = L10nRegistry::default();
        let mock_ff = MockFileFetcher::new(
            self.files
                .iter()
                .map(|p| (p.clone(), "".to_string()))
                .collect(),
        );

        let fs = self
            .sources
            .iter()
            .map(|s| {
                FileSource::new(
                    s.0.clone(),
                    s.1.iter().map(|l| l.parse().unwrap()).collect(),
                    s.2.clone(),
                    mock_ff.clone(),
                )
            })
            .collect();

        reg.register_sources(fs).unwrap();
        reg
    }
}
