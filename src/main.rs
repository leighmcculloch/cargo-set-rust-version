//! Update Cargo.toml rust-version to latest.
//!
//! If the given manifest is a workspace, all members are updated.
//!
//! # Install
//!
//! ```
//! cargo install --locked cargo-set-rust-version
//! ```
//!
//! # Usage
//!
//! Update the `rust-version` to the latest stable version:
//!
//! ```
//! cargo set-rust-version
//! ```

#![allow(clippy::missing_errors_doc)]

use clap::{AppSettings, Parser};
use std::fs;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[clap(
    version,
    about,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(bin_name = "cargo")]
enum RootCmd {
    SetRustVersion(SetRustVersionCmd),
}

#[derive(Parser, Debug)]
#[clap(version, about)]
struct SetRustVersionCmd {
    /// Cargo.toml file path
    #[clap(long, parse(from_os_str), default_value("Cargo.toml"))]
    manifest: std::path::PathBuf,
    /// Channel to use latest version
    #[clap(long, default_value("stable"))]
    channel: String,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("reading manifest")]
    ReadingManifest(io::Error),
    #[error("parsing manifest")]
    ParsingManifest(toml_edit::TomlError),

    #[error("parsing manifest workspace members is missing")]
    WorkspaceMembersIsMissing,
    #[error("parsing manifest workspace members is not array")]
    WorkspaceMembersIsNotArray,
    #[error("parsing manifest workspace member is not string")]
    WorkspaceMemberIsNotString,

    #[error("making http request")]
    Http(#[from] ureq::Error),

    #[error("making http request not string")]
    ParsingReleaseInfoNotString,
    #[error("parsing release info not valid toml")]
    ParsingReleaseInfoNotValidToml(#[from] toml::de::Error),
    #[error("parsing release info pkg section is missing")]
    ReleaseInfoPkgSectionIsMissing,
    #[error("parsing release info rustc section is missing")]
    ReleaseInfoRustcSectionIsMissing,
    #[error("parsing release info rustc version is missing")]
    ReleaseInfoRustCVersionIsMissing,
    #[error("parsing release info rustc version is not string")]
    ReleaseInfoRustCVersionIsNotString,
    #[error("parsing release info rustc version is empty")]
    ReleaseInfoRustCVersionIsEmpty,

    #[error("writing manifrst")]
    WritingManifest(io::Error),
}

impl SetRustVersionCmd {
    pub fn run(&self) -> Result<(), Error> {
        // Collect latest rust-version.
        println!("channel: {}", self.channel);
        let latest_version = {
            let url = format!(
                "https://static.rust-lang.org/dist/channel-rust-{}.toml",
                self.channel
            );
            let body = ureq::get(&url)
                .call()?
                .into_string()
                .map_err(|_| Error::ParsingReleaseInfoNotString)?;
            let info: toml::Value = toml::from_str(&body)?;
            let version_and_meta = info
                .get("pkg")
                .ok_or(Error::ReleaseInfoPkgSectionIsMissing)?
                .get("rustc")
                .ok_or(Error::ReleaseInfoRustcSectionIsMissing)?
                .get("version")
                .ok_or(Error::ReleaseInfoRustCVersionIsMissing)?
                .as_str()
                .ok_or(Error::ReleaseInfoRustCVersionIsNotString)?;
            let version = version_and_meta
                .split(' ')
                .next()
                .ok_or(Error::ReleaseInfoRustCVersionIsEmpty)?;
            let major_minor_version = version.split('.').take(2).collect::<Vec<_>>().join(".");
            major_minor_version
        };
        println!("latest rust-version: {}", latest_version);

        self.run_for_manifest(&self.manifest, &latest_version)
    }

    pub fn run_for_manifest(
        &self,
        manifest_path: impl AsRef<std::path::Path>,
        latest_version: &str,
    ) -> Result<(), Error> {
        let manifest_path_str = manifest_path.as_ref().to_string_lossy();
        println!("{}: reading", manifest_path_str);
        let manifest_raw = fs::read_to_string(&manifest_path).map_err(Error::ReadingManifest)?;
        let mut manifest = manifest_raw
            .parse::<toml_edit::Document>()
            .map_err(Error::ParsingManifest)?;

        // Check if workspace, and recursively load member manifests if so.
        if let Some(workspace) = manifest.get("workspace") {
            println!("{}: found workspace", manifest_path_str);
            let workspace_path = manifest_path
                .as_ref()
                .parent()
                .unwrap_or_else(|| manifest_path.as_ref());
            let members = workspace
                .get("members")
                .ok_or(Error::WorkspaceMembersIsMissing)?
                .as_array()
                .ok_or(Error::WorkspaceMembersIsNotArray)?;
            for m in members {
                let m_path = workspace_path
                    .join(m.as_str().ok_or(Error::WorkspaceMemberIsNotString)?)
                    .join("Cargo.toml");
                self.run_for_manifest(m_path, latest_version)?;
            }
            return Ok(());
        }

        // Collect current rust-version.
        let current_version = manifest
            .get("package")
            .and_then(|package| package.get("rust-version"))
            .and_then(toml_edit::Item::as_str);

        // If current and latest are same, do nothing.
        if let Some(current_version) = current_version {
            if current_version == latest_version {
                println!(
                    "{}: up-to-date rust-version: {}",
                    manifest_path_str, current_version
                );
                return Ok(());
            }
        }

        // Update rust-version to latest.
        println!(
            "{}: updating rust-version: {} => {}",
            manifest_path_str,
            current_version.unwrap_or("None"),
            latest_version
        );
        manifest["package"]["rust-version"] = toml_edit::value(latest_version);
        fs::OpenOptions::new()
            .write(true)
            .open(&manifest_path)
            .map_err(Error::WritingManifest)?
            .write_all(manifest.to_string().as_bytes())
            .map_err(Error::WritingManifest)?;

        Ok(())
    }
}

fn main() {
    if let Err(e) = match RootCmd::parse() {
        RootCmd::SetRustVersion(cmd) => cmd.run(),
    } {
        eprintln!("error: {}", e);
    }
}
