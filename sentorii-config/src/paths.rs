//! Provides testable abstractions over the process's file paths and project discovery.

use std::io;
use std::path::PathBuf;
use std::process::Command;

const CONFIG_FILE_NAME: &str = "config.toml";

fn find_git_repo_root() -> io::Result<Option<PathBuf>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(PathBuf::from(stdout)))
        }
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

/// A trait that abstracts the discovery of the project-level configuration file.
pub trait ProjectPathProvider {
    /// Discovers the full path to the project-level `sentorii.toml` file.
    /// # Errors
    /// Errors could arise from the underhood git call being made.
    fn project_config_path(&self) -> io::Result<Option<PathBuf>>;
}

/// The production `ProjectPathProvider` that uses `git` to discover the project root.
pub struct SystemProjectPathProvider;

impl ProjectPathProvider for SystemProjectPathProvider {
    fn project_config_path(&self) -> io::Result<Option<PathBuf>> {
        find_git_repo_root()?.map_or(Ok(None), |root| {
            Ok(Some(root.join(".sentorii").join(CONFIG_FILE_NAME)))
        })
    }
}

/// A mock `ProjectPathProvider` for use in tests.
#[cfg(test)]
pub struct MockProjectPathProvider {
    pub path: io::Result<Option<PathBuf>>,
}

#[cfg(test)]
impl ProjectPathProvider for MockProjectPathProvider {
    fn project_config_path(&self) -> io::Result<Option<PathBuf>> {
        match &self.path {
            Ok(Some(p)) => Ok(Some(p.clone())),
            Ok(None) => Ok(None),
            Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
        }
    }
}

/// A trait that abstracts the discovery of the user-global configuration file.
pub trait GlobalPathProvider {
    /// Provides the full path to the global `sentorii.toml` file.
    fn global_config_path(&self) -> Option<PathBuf>;
}

/// The production `GlobalPathProvider` that uses the `directories` crate.
pub struct SystemGlobalPathProvider {
    home_dir: Option<PathBuf>,
}

impl SystemGlobalPathProvider {
    #[must_use]
    pub const fn new(home_dir: Option<PathBuf>) -> Self {
        Self { home_dir }
    }
}

impl GlobalPathProvider for SystemGlobalPathProvider {
    fn global_config_path(&self) -> Option<PathBuf> {
        self.home_dir
            .as_ref()
            .map(|home| home.join(".config").join("sentorii").join(CONFIG_FILE_NAME))
    }
}

/// A mock `GlobalPathProvider` for use in tests.
#[cfg(test)]
pub struct MockGlobalPathProvider {
    pub path: Option<PathBuf>,
}

#[cfg(test)]
impl GlobalPathProvider for MockGlobalPathProvider {
    fn global_config_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}
