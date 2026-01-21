use std::fs::{create_dir, write};
use std::path::Path;
use assert_cmd::cargo_bin_cmd;
use predicates::prelude::predicate;
use tempfile::tempdir;

fn setup_package(dir: &Path, toml_content: &str) -> Result<(), std::io::Error> {
    let src_path = dir.join("src");
    create_dir(&src_path)?;
    write(src_path.join("lib.rs"), "")?;
    write(dir.join("Cargo.toml"), toml_content)?;
    Ok(())
}

#[test]
fn test_get_info() -> Result<(), Box<dyn std::error::Error>> {
    // ARRANGE: A valid package is needed for the loader to succeed.
    let dir = tempdir()?;
    setup_package(dir.path(), "[package]\nname = \"info\"\nversion = \"0.1.0\"")?;

    // ACT
    let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
    cmd.current_dir(dir.path());
    cmd.write_stdin(r#"{"command":"get-info"}"#.to_string() + "\n");

    // ASSERT: Check that the static info is returned correctly.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""kind":"info""#))
        .stdout(predicate::str::contains(r#""plugin_name":"Cargo""#));

    Ok(())
}

mod standalone_crate {
    use std::fs::read_to_string;
    use super::*;

    #[test]
    fn get_version() -> Result<(), Box<dyn std::error::Error>> {
        // ARRANGE
        let dir = tempdir()?;
        setup_package(
            dir.path(),
            r#"
[package]
name = "my-crate"
version = "0.1.0"
"#,
        )?;

        // ACT
        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path());
        cmd.write_stdin(r#"{"command":"get-version"}"#.to_string() + "\n");

        // ASSERT
        cmd.assert().success().stdout(predicate::str::contains(
            r#"{"kind":"version","version":"0.1.0"}"#,
        ));

        Ok(())
    }

    #[test]
    fn set_version_preserves_formatting() -> Result<(), Box<dyn std::error::Error>> {
        // ARRANGE
        let dir = tempdir()?;
        let mock_toml = r#"
# Main package definition
[package]
name    = "my-crate"
# The version to be changed
version = "1.2.3"
edition = "2021"
"#;
        setup_package(dir.path(), mock_toml)?;

        // ACT
        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path());
        cmd.write_stdin(
            r#"{"command":"set-version","payload":{"version":"2.0.0"}}"#.to_string() + "\n",
        );

        // ASSERT
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(r#"{"kind":"ack"}"#));

        let updated_toml = read_to_string(dir.path().join("Cargo.toml"))?;
        let expected_toml = r#"
# Main package definition
[package]
name    = "my-crate"
# The version to be changed
version = "2.0.0"
edition = "2021"
"#;
        assert_eq!(updated_toml, expected_toml);

        Ok(())
    }
}

mod workspace_project {
    use std::fs::read_to_string;
    use tempfile::TempDir;
    use super::*;

    // Helper to set up a standard workspace for testing.
    fn setup_workspace(dir: &TempDir) -> Result<(), std::io::Error> {
        let root_toml = r#"
# Workspace Root
[workspace]
members = ["member_a", "member_b"]

[workspace.package]
version = "0.5.0"
"#;
        write(dir.path().join("Cargo.toml"), root_toml)?;

        // member_a inherits its version
        let member_a_path = dir.path().join("member_a");
        create_dir(&member_a_path)?;
        setup_package(&member_a_path, r#"
[package]
name = "member_a"
version = { workspace = true }
"#)?;

        // member_b has its own version
        let member_b_path = dir.path().join("member_b");
        create_dir(&member_b_path)?;
        setup_package(&member_b_path, r#"
[package]
name = "member_b"
version = { workspace = true }
"#)?;
        Ok(())
    }

    #[test]
    fn get_version_from_workspace_root() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        setup_workspace(&dir)?;

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path()); // Running from the root
        cmd.write_stdin(r#"{"command":"get-version"}"#.to_string() + "\n");

        // It should find the workspace version.
        cmd.assert().success().stdout(predicate::str::contains(
            r#"{"kind":"version","version":"0.5.0"}"#,
        ));

        Ok(())
    }

    #[test]
    fn get_version_from_inheriting_member() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        setup_workspace(&dir)?;

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path().join("member_a")); // Running from the member
        cmd.write_stdin(r#"{"command":"get-version"}"#.to_string() + "\n");

        // The plugin should still correctly report the inherited version.
        cmd.assert().success().stdout(predicate::str::contains(
            r#"{"kind":"version","version":"0.5.0"}"#,
        ));

        Ok(())
    }

    #[test]
    fn set_version_from_member_correctly_modifies_root() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        setup_workspace(&dir)?;

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path().join("member_a")); // Running from the member
        cmd.write_stdin(
            r#"{"command":"set-version","payload":{"version":"0.6.0"}}"#.to_string() + "\n",
        );

        cmd.assert()
            .success()
            .stdout(predicate::str::contains(r#"{"kind":"ack"}"#));

        // ASSERT: The ROOT Cargo.toml was modified.
        let updated_root_toml = read_to_string(dir.path().join("Cargo.toml"))?;
        assert!(updated_root_toml.contains(r#"version = "0.6.0""#));

        Ok(())
    }
}

mod error_conditions {
    use super::*;

    #[test]
    fn no_manifest_fails_gracefully() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?; // An empty directory

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path());
        cmd.write_stdin(r#"{"command":"get-version"}"#.to_string() + "\n");

        // ASSERT: The loader should fail and the PDK wrapper should report the error.
        cmd.assert()
            .failure()
            .stdout(predicate::str::contains(r#""kind":"error""#))
            .stdout(predicate::str::contains(r#""code":"PLUGIN_LOGIC_FAILED""#));

        Ok(())
    }

    #[test]
    fn malformed_manifest_fails_gracefully() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        write(dir.path().join("Cargo.toml"), "this is not valid toml")?;

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path());
        cmd.write_stdin(r#"{"command":"get-version"}"#.to_string() + "\n");

        cmd.assert()
            .failure()
            .stdout(predicate::str::contains(r#""kind":"error""#))
            .stdout(predicate::str::contains(r#""code":"PLUGIN_LOGIC_FAILED""#));

        Ok(())
    }

    #[test]
    fn set_version_with_invalid_semver_fails_gracefully() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        setup_package(dir.path(), "[package]\nname = \"semver\"\nversion = \"0.1.0\"")?;

        let mut cmd = cargo_bin_cmd!("sentorii-plugin-cargo");
        cmd.current_dir(dir.path());
        cmd.write_stdin(
            r#"{"command":"set-version","payload":{"version":"not-a-version"}}"#.to_string() + "\n",
        );

        cmd.assert()
            .success() // The plugin itself doesn't crash, it handles the error.
            .stdout(predicate::str::contains(r#""status":"error""#))
            .stdout(predicate::str::contains(r#""code":"INVALID_INPUT""#));

        Ok(())
    }
}