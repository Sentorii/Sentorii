use cargo_metadata::Error;
use sentorii_pdk::error::PdkError;
use thiserror::Error;
use toml_edit::TomlError;

#[derive(Error, Debug)]
pub enum VersionValidatorError {
    #[error(
        "Version contains illegal characters: slashes ('/') are not allowed in Cargo versions."
    )]
    IllegalSlash,

    #[error("Invalid semver format: {0}. Expected MAJOR.MINOR.PATCH.")]
    InvalidSemver(String),
}

pub fn toml_error_to_pdk_error(err: TomlError) -> PdkError {
    PdkError::Io(std::io::Error::other(format!("{err}")))
}

pub fn cargo_metadata_error_to_pdk_error(err: Error) -> PdkError {
    PdkError::PluginLogic(err.to_string())
}
