use crate::error::{cargo_metadata_error_to_pdk_error};
use crate::manifest::ManifestFile;
use crate::plugin::CargoPlugin;
use cargo_metadata::MetadataCommand;
use sentorii_pdk::error::PdkError;

pub struct Loader;

impl Loader {
    pub fn load() -> Result<CargoPlugin, PdkError> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .map_err(cargo_metadata_error_to_pdk_error)?;
        
        let root_path = metadata.workspace_root.join("Cargo.toml").into_std_path_buf();
        let root = ManifestFile::open(root_path)?;
        
        let mut members = Vec::new();
        let mut metadata_versions = Vec::new();
        
        for package in &metadata.workspace_members {
            let pkg = &metadata[package];
            metadata_versions.push(pkg.version.to_string());
            
            if pkg.manifest_path != metadata.workspace_root.join("Cargo.toml") {
                members.push(ManifestFile::open(pkg.manifest_path.clone().into_std_path_buf())?);
            }
        }
        
        Ok(CargoPlugin {
            root,
            members,
            metadata_versions,
        })
    }
}