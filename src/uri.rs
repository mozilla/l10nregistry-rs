use std::ffi::OsString;
use std::path::{Path, PathBuf};
use unic_langid::LanguageIdentifier;

pub trait ResourceURI {
    fn resolve_path(&self, langid: &LanguageIdentifier) -> PathBuf;
}

impl ResourceURI for Path {
    fn resolve_path(&self, langid: &LanguageIdentifier) -> PathBuf {
        let replacement: OsString = langid.to_string().into();
        self.iter()
            .map(|part| {
                if part == "{locale}" {
                    replacement.as_os_str()
                } else {
                    part
                }
            })
            .collect()
    }
}
