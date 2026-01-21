use crate::error::toml_error_to_pdk_error;
use sentorii_pdk::error::PdkError;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use toml_edit::DocumentMut;

pub struct ManifestFile {
    pub path: PathBuf,
    pub doc: DocumentMut,
}

impl ManifestFile {
    pub fn open(path: PathBuf) -> Result<Self, PdkError> {
        let content = read_to_string(&path).map_err(PdkError::Io)?;
        let doc = content
            .parse::<DocumentMut>()
            .map_err(toml_error_to_pdk_error)?;
        Ok(Self { path, doc })
    }

    pub fn get_package_version(&self) -> Option<String> {
        self.doc
            .get("package")?
            .get("version")?
            .as_str()
            .map(|s| s.to_string())
    }

    pub fn is_inheriting_version(&self) -> bool {
        self.doc
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.get("workspace"))
            .and_then(|w| w.as_bool())
            == Some(true)
    }

    pub fn get_workspace_version(&self) -> Option<String> {
        self.doc
            .get("workspace")?
            .get("package")?
            .get("version")?
            .as_str()
            .map(|s| s.to_string())
    }

    pub fn save(&self) -> Result<(), PdkError> {
        write(&self.path, self.doc.to_string())?;
        Ok(())
    }
}
