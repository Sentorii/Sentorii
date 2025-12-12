#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use sentorii_config::{Config, ConfigError, load_config};
use std::env;
use std::path::{Path, PathBuf};

struct ChangeDir {
    original: PathBuf,
}

impl ChangeDir {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let original = env::current_dir().expect("Failed to get current directory");
        env::set_current_dir(path.into()).expect("Failed to change directory");
        Self { original }
    }
}

impl Drop for ChangeDir {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).expect("Failed to restore original directory");
    }
}

fn in_dir<F>(path: &Path, closure: F)
where
    F: FnOnce(),
{
    let _guard = ChangeDir::new(path);
    closure();
}

fn setup_git_repo(dir: &TempDir) {
    Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(dir.path())
        .assert()
        .success();
}

fn setup_global_config(fake_home: &TempDir, content: &str) {
    let global_dir = fake_home.child(".config/sentorii");
    global_dir.create_dir_all().unwrap();
    global_dir.child("config.toml").write_str(content).unwrap();
}

fn setup_project_config(repo_root: &TempDir, content: &str) {
    let project_dir = repo_root.child(".sentorii");
    project_dir.create_dir_all().unwrap();
    project_dir.child("config.toml").write_str(content).unwrap();
}

#[test]
fn test_full_hierarchy() {
    let temp_repo = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    setup_git_repo(&temp_repo);
    setup_global_config(
        &fake_home,
        r#"
[branching]
main = "from-global"
"#,
    );
    setup_project_config(
        &temp_repo,
        r#"
[branching]
main = "from-project"
develop = "from-project"
"#,
    );

    in_dir(temp_repo.path(), || {
        let vars_to_set = vec![
            ("HOME", Some(fake_home.path().to_str().unwrap())),
            ("SENTORII_BRANCHING_MAIN", Some("from-env")),
        ];

        temp_env::with_vars(vars_to_set, || {
            let config = load_config().unwrap();

            assert_eq!(config.branching.main, "from-env");
            assert_eq!(config.branching.develop, "from-project");
            assert_eq!(config.branching.prefixes.feature, "feature/");
        });
    });
}

#[test]
fn test_no_project_config_is_ok() {
    let temp_repo = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    setup_git_repo(&temp_repo);

    in_dir(temp_repo.path(), || {
        temp_env::with_var("HOME", Some(fake_home.path().to_str().unwrap()), || {
            let config = load_config().unwrap();
            assert_eq!(config, Config::default());
        });
    });
}

#[test]
fn test_malformed_project_file_returns_error() {
    let temp_repo = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    setup_git_repo(&temp_repo);
    setup_project_config(&temp_repo, "this is not a valid toml");

    in_dir(temp_repo.path(), || {
        temp_env::with_var("HOME", Some(fake_home.path().to_str().unwrap()), || {
            let result = load_config();

            assert!(result.is_err());
            match result.unwrap_err() {
                ConfigError::TomlParseError { path, .. } => {
                    assert_eq!(path, temp_repo.path().join(".sentorii").join("config.toml"));
                }
                other => panic!("Expected Toml error, got {other:?}"),
            }
        });
    });
}
