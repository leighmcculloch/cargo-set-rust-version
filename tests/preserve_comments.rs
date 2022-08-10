use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn preserve_comments() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = assert_fs::NamedTempFile::new("Cargo.toml")?;
    manifest.write_str(
        r#"
[package]
# Keep rust-version up to date.
rust-version = "1.61"
"#,
    )?;

    let mut cmd = Command::cargo_bin("cargo-set-rust-version")?;
    cmd.arg("set-rust-version");
    cmd.arg("--manifest").arg(manifest.path());
    cmd.arg("--channel").arg("1.62");
    cmd.assert().success().stdout(format!(
        "manifest file: {}
current rust-version: 1.61
channel: 1.62
latest rust-version: 1.62
updating rust-version: 1.61 => 1.62
",
        manifest.path().to_string_lossy()
    ));

    manifest.assert(
        r#"
[package]
# Keep rust-version up to date.
rust-version = "1.62"
"#,
    );

    Ok(())
}
