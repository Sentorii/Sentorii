use crate::manifest::ManifestFile;
use sentorii_pdk::Plugin;
use sentorii_pdk::error::PdkError;
use sentorii_pdk::sentorii_api::{PluginInfo, SetVersionPayload, VersionResponse};
use std::collections::HashSet;
use toml_edit::value;

pub struct CargoPlugin {
    pub root: ManifestFile,
    pub members: Vec<ManifestFile>,
    pub metadata_versions: Vec<String>,
}

impl Plugin for CargoPlugin {
    fn get_info(&mut self) -> Result<PluginInfo, PdkError> {
        Ok(PluginInfo {
            plugin_name: "Cargo".to_string(),
            binary_name: "sentorii-plugin-cargo".to_string(),
            primary_file: "Cargo.toml".to_string(),
            hint_files: vec!["Cargo.lock".to_string()],
            permissions: Default::default(),
        })
    }

    fn get_version(&mut self) -> Result<VersionResponse, PdkError> {
        let mut versions = HashSet::new();

        if let Some(v) = self.root.get_package_version() {
            versions.insert(v);
        }
        if let Some(v) = self.root.get_workspace_version() {
            versions.insert(v);
        }

        for member in &self.members {
            if let Some(v) = member.get_package_version() {
                versions.insert(v);
            }
        }

        if versions.len() > 1 {
            return Err(PdkError::PluginLogic(format!(
                "Conflicting versions found in project: {:?}",
                versions
            )));
        }

        let version = versions
            .into_iter()
            .next()
            .ok_or_else(|| PdkError::PluginLogic("No version found in project".to_string()))?;

        Ok(VersionResponse { version })
    }

    fn set_version(&mut self, payload: SetVersionPayload) -> Result<(), PdkError> {
        let version = payload.version;
        if self.root.get_package_version().is_some() {
            self.root.doc["package"]["version"] = value(&version);
        }
        if self.root.get_workspace_version().is_some() {
            self.root.doc["workspace"]["package"]["version"] = value(&version);
        }
        self.root.save()?;

        for member in &mut self.members {
            if !member.is_inheriting_version() && member.get_package_version().is_some() {
                member.doc["package"]["version"] = value(&version);
                member.save()?;
            }
        }

        Ok(())
    }
}
