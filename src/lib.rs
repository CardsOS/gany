use anyhow::Context;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::option::Option;
use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// The data associated with a package
pub struct Package {
    /// The CPU architecture intended for use with the packaged binaries
    pub arch: String,
    /// The description of the package
    pub description: String,
    /// The version of the packaged software
    pub version: String,
    /// The package names, along with their versions, that a package depends on
    pub dependencies: Option<Vec<(String, String)>>,
    /// The files that a package owns, including potential ghost files
    pub files: Vec<String>,
    /// The SHA3-256 hash of the LZ4-compressed archive the software is packaged in
    pub keccak: String,
}

/// Add a package to your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Path to a Mokk (required)
pub fn add_package(matches: &clap::ArgMatches) {
    let package_name = matches
        .value_of("PACKAGE_NAME")
        .with_context(|| "No package name was given".to_string())
        .unwrap();
}

/// Drop a package from your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Path to a Mokk (required)
pub fn drop_package(matches: &clap::ArgMatches) {
    let package_name = matches
        .value_of("PACKAGE_NAME")
        .with_context(|| "No package name was given".to_string())
        .unwrap();
}

/// Refresh the local package repository with one from a remote software distribution
pub fn refresh_remote_packages() {}

/// Upgrade your local packages with newer versions held in a remote software distribution
pub fn upgrade_packages() {}

/// Package a piece of software for future distribution
///
/// # Arguments
///
/// * `PATH` - Path to a Mokk (required)
pub fn create_package(matches: &clap::ArgMatches) {
    let path = matches
        .value_of("PATH")
        .with_context(|| "No path to a Mokk was given".to_string())
        .unwrap();
    env::set_current_dir(path)
        .with_context(|| format!("Could not read a Mokk at {}", path))
        .unwrap();
}

/// Write a file to the filesystem
///
/// # Arguments
///
/// * `path` - The path to write the file to
///
/// * `data_to_write` - The data to write to the filesystem
#[inline(always)]
pub fn write_file(path: &str, data_to_write: &[u8]) {
    fs::create_dir_all(Path::new(path).parent().unwrap()).unwrap(); // Create output path, write to file
    let file = File::create(&path).unwrap(); // Create file which we will write to
    let mut buffered_writer = BufWriter::new(file); // Create a buffered writer, allowing us to modify the file we've just created
    buffered_writer
        .write_all(data_to_write)
        .with_context(|| format!("Could not write data to {}", path))
        .unwrap(); // Write data to file
    buffered_writer.flush().unwrap(); // Empty out the data from memory after we've written to the file
}