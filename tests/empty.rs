use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn empty() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = assert_fs::NamedTempFile::new("Cargo.toml")?;
    manifest.write_str("")?;

    let mut cmd = Command::cargo_bin("cargo-set-rust-version")?;
    cmd.arg("set-rust-version");
    cmd.arg("--manifest").arg(manifest.path());
    cmd.arg("--channel").arg("1.62");
    cmd.assert().success().stdout(format!(
        "channel: 1.62
latest rust-version: 1.62
{0}: reading
",
        manifest.path().to_string_lossy()
    ));

    manifest.assert("");

    Ok(())
}
