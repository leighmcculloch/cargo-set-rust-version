use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn nothing_to_do() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = assert_fs::NamedTempFile::new("Cargo.toml")?;
    manifest.write_str(
        r#"
[package]
rust-version = "1.62"
"#,
    )?;

    let mut cmd = Command::cargo_bin("cargo-set-rust-version")?;
    cmd.arg("set-rust-version");
    cmd.arg("--manifest").arg(manifest.path());
    cmd.arg("--channel").arg("1.62");
    cmd.assert().success().stdout(format!(
        "channel: 1.62
latest rust-version: 1.62
manifest file: {}
up-to-date rust-version: 1.62
",
        manifest.path().to_string_lossy()
    ));

    manifest.assert(
        r#"
[package]
rust-version = "1.62"
"#,
    );

    Ok(())
}
