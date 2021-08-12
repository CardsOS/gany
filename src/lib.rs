use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::convert::TryInto;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::option::Option;
use std::path::Path;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// The data associated with a package
pub struct Package {
    /// The CPU architecture intended for use with the packaged binaries
    pub arch: String,
    /// The name of the package
    pub name: String,
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
        .with_context(|| "No path to a package source was given".to_string())
        .unwrap();
    env::set_current_dir(path)
        .with_context(|| format!("Could not read a package source at {}", path))
        .unwrap();

    let package_source_path = format!("{}/src/", path);
    let human_package_meta_file_path = format!("{}/manifest.yaml", path);
    let human_package_meta_file = fs::read_to_string(human_package_meta_file_path)
        .expect("Unable to read package information");
    let mut package_object: Package = serde_yaml::from_str(&human_package_meta_file)
        .expect("Unable to process package information");
    let bin_package_meta_output_path = format!("{}/out/{}.ganyinf", path, package_object.name);
    let package_output_path = format!("{}/out/{}.gany", path, package_object.name);

    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all(".", package_source_path)
        .expect("Failed to write archive");
    tar.finish().expect("Unable to finish writing archive");
    let tar_bytes: &Vec<u8> = tar.get_ref();
    let package_archive_file = lz4_flex::compress_prepend_size(tar_bytes);
    write_file(&package_output_path, &package_archive_file);

    let package_archive_hash = sha3::Sha3_256::digest(&package_archive_file);
    package_object.keccak = format!("{:x}", package_archive_hash);

    let bin_package_object = bincode::serialize(&package_object).unwrap();
    write_file(&bin_package_meta_output_path, &bin_package_object);

    let uncompressed_size_bytes: [u8; 4] = package_archive_file[0..4].try_into().unwrap();
    let uncompressed_size: u32 = u32::from_le_bytes(uncompressed_size_bytes);
    println!(
        "Wrote package '{}' to filesystem (path:{}, size: {} bytes, SHA3-256: {:x})",
        package_object.name, package_output_path, uncompressed_size, package_archive_hash
    );
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
