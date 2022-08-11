use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn workspace() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let manifest = temp.child("Cargo.toml");
    manifest.write_str(
        r#"
[workspace]
members = ["a", "b"]
"#,
    )?;
    let manifest_a = temp.child("a/Cargo.toml");
    manifest_a.write_str(
        r#"
[package]
rust-version = "1.60"
"#,
    )?;
    let manifest_b = temp.child("b/Cargo.toml");
    manifest_b.write_str(
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
{0}: reading
{0}: found workspace
{1}: reading
{1}: updating rust-version: 1.60 => 1.62
{2}: reading
{2}: up-to-date rust-version: 1.62
",
        manifest.path().to_string_lossy(),
        manifest_a.path().to_string_lossy(),
        manifest_b.path().to_string_lossy(),
    ));

    manifest.assert(
        r#"
[workspace]
members = ["a", "b"]
"#,
    );
    manifest_a.assert(
        r#"
[package]
rust-version = "1.62"
"#,
    );
    manifest_b.assert(
        r#"
[package]
rust-version = "1.62"
"#,
    );

    Ok(())
}
