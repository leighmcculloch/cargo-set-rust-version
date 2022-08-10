#![allow(clippy::items_after_statements)]

use clap::{AppSettings, Parser};
use std::fs;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[clap(version, disable_help_subcommand = true, disable_version_flag = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Root {
    /// Cargo.toml file path
    #[clap(long, parse(from_os_str), default_value("Cargo.toml"))]
    manifest: std::path::PathBuf,
    /// Channel to use latest version (stable, nightly)
    #[clap(long, default_value("stable"))]
    channel: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reading manifrst")]
    ReadingManifest(io::Error),
    #[error("parsing manifest")]
    ParsingManifest(toml_edit::TomlError),

    #[error("making http request")]
    Reqwest(#[from] reqwest::Error),

    #[error("parsing release info")]
    ParsingReleaseInfo(#[from] toml::de::Error),

    #[error("writing manifrst")]
    WritingManifest(io::Error),
}

impl Root {
    pub fn run(&self) -> Result<(), Error> {
        // Collect current rust-version.
        println!("manifest file: {}", self.manifest.to_string_lossy());
        let manifest_raw = fs::read_to_string(&self.manifest).map_err(Error::ReadingManifest)?;
        let mut manifest = manifest_raw
            .parse::<toml_edit::Document>()
            .map_err(Error::ParsingManifest)?;
        let current_version = manifest
            .get("package")
            .and_then(|package| package.get("rust-version"))
            .and_then(toml_edit::Item::as_str);
        println!(
            "current rust-version: {}",
            current_version.unwrap_or("None")
        );

        // Collect latest rust-version.
        println!("channel: {}", self.channel);
        let latest_version = {
            let url = format!(
                "https://static.rust-lang.org/dist/channel-rust-{}.toml",
                self.channel
            );
            let resp = reqwest::blocking::get(url)?;
            let bytes = &resp.bytes()?;
            let info: toml::Value = toml::from_slice(bytes)?;
            let version_and_meta = info["pkg"]["rustc"]["version"].as_str().unwrap();
            let version = version_and_meta.split(' ').next().unwrap();
            let major_minor_version = version.split('.').take(2).collect::<Vec<_>>().join(".");
            major_minor_version
        };
        println!("latest rust-version: {}", latest_version);

        // If current and latest are same, do nothing.
        if let Some(current_version) = current_version {
            if current_version == latest_version {
                return Ok(());
            }
        }

        // Update rust-version to latest.
        println!(
            "updating version: {} => {}",
            current_version.unwrap_or("None"),
            latest_version
        );
        manifest["package"]["rust-version"] = toml_edit::value(latest_version);
        fs::OpenOptions::new()
            .write(true)
            .open(&self.manifest)
            .map_err(Error::WritingManifest)?
            .write_all(manifest.to_string().as_bytes())
            .map_err(Error::WritingManifest)?;

        Ok(())
    }
}

fn main() {
    let root = Root::parse();
    if let Err(e) = root.run() {
        eprintln!("error: {:?}", e);
    }
}
