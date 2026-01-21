use cargo_metadata::Error;
use toml_edit::TomlError;
use sentorii_pdk::error::PdkError;

pub fn toml_error_to_pdk_error(err: TomlError) -> PdkError {
    PdkError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))
}

pub fn cargo_metadata_error_to_pdk_error(err: Error) -> PdkError {
    PdkError::PluginLogic(err.to_string())
}