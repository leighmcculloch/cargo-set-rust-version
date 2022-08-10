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
    #[error("loading manifest")]
    TomlEdit(#[from] toml_edit::TomlError),
    #[error("missing [package] section in manifest")]
    MissingPackage,
    #[error("must be string")]
    MustBeString,

    #[error("making http request")]
    Reqwest(#[from] reqwest::Error),

    #[error("parsing channel info")]
    TomlDe(#[from] toml::de::Error),

    #[error("writing manifest")]
    TomlSer(#[from] toml::ser::Error),
    #[error("writing file")]
    Io(#[from] io::Error),
}

impl Root {
    pub fn run(&self) -> Result<(), Error> {
        // Collect current rust-version.
        println!("manifest file: {}", self.manifest.to_string_lossy());
        let manifest_raw = fs::read_to_string(&self.manifest)?;
        let mut manifest = manifest_raw.parse::<toml_edit::Document>()?;
        let package = manifest.get("package").ok_or(Error::MissingPackage)?;
        let current_version = package
            .get("rust-version")
            .map(|item| item.as_str().ok_or(Error::MustBeString))
            .transpose()?;
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
            .open(&self.manifest)?
            .write_all(manifest.to_string().as_bytes())?;

        Ok(())
    }
}

fn main() {
    let root = Root::parse();
    if let Err(e) = root.run() {
        eprintln!("error: {:?}", e);
    }
}
