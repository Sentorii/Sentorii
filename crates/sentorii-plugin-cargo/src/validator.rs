use crate::error::VersionValidatorError;
use cargo_metadata::semver::Version;

pub struct VersionValidator;

impl VersionValidator {
    pub fn validate(version_str: &str) -> Result<(), VersionValidatorError> {
        if version_str.contains('/') {
            return Err(VersionValidatorError::IllegalSlash);
        }

        let _v = Version::parse(version_str)
            .map_err(|e| VersionValidatorError::InvalidSemver(e.to_string()))?;

        Ok(())
    }
}
