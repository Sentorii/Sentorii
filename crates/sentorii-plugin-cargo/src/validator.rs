use cargo_metadata::semver::Version;
use crate::error::VersionValidatorError;

pub struct VersionValidator;

impl VersionValidator {
    pub fn validate(version_str: &String) -> Result<(), VersionValidatorError> {
        if version_str.contains('/') {
            return Err(VersionValidatorError::IllegalSlash);
        }

        let v = Version::parse(version_str.as_str())
            .map_err(|e| VersionValidatorError::InvalidSemver(e.to_string()))?;

        Ok(())
    }
}