#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::Command;
use assert_fs::prelude::*;
use sentorii_config::env::{EnvironmentProvider, ProcessEnvironmentProvider};
use sentorii_config::parser::{SystemValueProvider, ValueProvider};
use sentorii_config::paths::{
    GlobalPathProvider, ProjectPathProvider, SystemGlobalPathProvider, SystemProjectPathProvider,
};
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use toml::{Table, Value};

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
        env::set_current_dir(&self.original).expect("Failed to change directory");
    }
}

#[test]
fn systemprojectpathprovider_finds_git_root_correctly() {
    let temp_repo = assert_fs::TempDir::new().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(temp_repo.path())
        .assert()
        .success();
    let subdir = temp_repo.child("src/app");
    subdir.create_dir_all().unwrap();
    let expected_config_path = temp_repo.path().join(".sentorii").join("config.toml");
    subdir.child(".sentorii").create_dir_all().unwrap();

    let _guard = ChangeDir::new(subdir.path());
    let provider = SystemProjectPathProvider;
    let found_path = provider.project_config_path().unwrap();

    assert_eq!(found_path, Some(expected_config_path));
}

#[test]
fn systemglobalpathprovider_finds_global_path_correctly() {
    let fake_home = assert_fs::TempDir::new().unwrap();
    let expected_global_path = fake_home.path().join(".config/sentorii/config.toml");

    temp_env::with_var("HOME", Some(fake_home.path()), || {
        let provider = SystemGlobalPathProvider::new(Some(fake_home.path().to_path_buf()));
        let found_path = provider.global_config_path();
        assert_eq!(found_path, Some(expected_global_path));
    });
}

#[test]
fn processenvironment_reads_from_real_environment() {
    let vars_to_set = vec![
        ("SENTORII_TEST_VAR_1", Some("value1")),
        ("SENTORII_TEST_VAR_2", Some("value2")),
    ];

    temp_env::with_vars(vars_to_set, || {
        let provider = ProcessEnvironmentProvider;
        let found_vars: BTreeMap<String, String> = provider.vars().collect();

        assert_eq!(found_vars.get("SENTORII_TEST_VAR_1").unwrap(), "value1");
        assert_eq!(found_vars.get("SENTORII_TEST_VAR_2").unwrap(), "value2");
    });
}

#[test]
fn systemvalueprovider_loads_and_parses_file_correctly() {
    let temp_file = assert_fs::NamedTempFile::new("config.toml").unwrap();
    let toml_content = r#"
[branching]
main = "live"
"#;
    temp_file.write_str(toml_content).unwrap();

    let provider = SystemValueProvider;
    let value_result = provider
        .load_from(Some(temp_file.path().to_path_buf()))
        .unwrap();

    let expected_table: Table = toml_content.parse().unwrap();
    let expected_value = Value::Table(expected_table);
    assert_eq!(value_result, Some(expected_value));
}
