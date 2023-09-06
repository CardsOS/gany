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
use std::path::PathBuf;

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
    pub dependencies: Option<Vec<(String, Option<String>)>>,
    /// The package names, along with their versions, that a package conflicts with
    pub conflicts: Option<Vec<(String, Option<String>)>>,
    /// The files that a package owns, including potential ghost files
    pub files: Vec<String>,
    /// The SHA3-256 hash of the LZ4-compressed archive the software is packaged in
    pub keccak: String,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// The data associated with a repository
pub struct Repository {
    /// The CPU architecture the repository supports
    pub arch: String,
    /// The name of the repository
    pub name: String,
    /// The description of the repository
    pub description: String,
    /// The address (URL or IP) of the repository
    pub address: String,
    /// The packages within this repository
    pub packages: Vec<Package>,
}

/// Add a package to your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Name of a package (required)
pub fn add_package(matches: &clap::ArgMatches) {
    let _package_name = matches
        .value_of("PACKAGE_NAME")
        .with_context(|| "No package name was given".to_string())
        .unwrap();
}

/// Drop a package from your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Name of a package (required)
pub fn drop_package(matches: &clap::ArgMatches) {
    let _package_name = matches
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
/// * `PATH` - Path to a package source (required)
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
    let bin_package_meta_output_path = format!(
        "{}/out/{}-{}-{}.ganyinf",
        path, package_object.arch, package_object.name, package_object.version
    );
    let package_output_path = format!(
        "{}/out/{}-{}-{}.gany",
        path, package_object.arch, package_object.name, package_object.version
    );

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
        "Wrote package '{}' to filesystem (path: {}, uncompressed size: {} bytes, SHA3-256: {:x})",
        package_object.name, package_output_path, uncompressed_size, package_archive_hash
    );
}

/// Extract a software package
///
/// # Arguments
///
/// * `PATH` - Path to a package file (required)
pub fn extract_package(matches: &clap::ArgMatches) {
    let path = matches
        .value_of("PATH")
        .with_context(|| "No path to a package was given".to_string())
        .unwrap();
    let path_buf = PathBuf::from(path);
    let package_file_name = path_buf.file_stem().unwrap().to_str().unwrap();
    let package_parent_directory = path_buf.parent().unwrap().to_str().unwrap();
    env::set_current_dir(package_parent_directory)
        .with_context(|| {
            format!(
                "Could not access parent directory of package file ({})",
                package_parent_directory
            )
        })
        .unwrap();

    let _package_file = File::open(path).expect("Could not open package file");
    let package_file_bytes = fs::read(path).expect("Could not read package file");
    let package_extract_path = format!(
        "{}/{}-contents/",
        package_parent_directory, package_file_name
    );

    let decompressed_package = lz4_flex::decompress_size_prepended(&package_file_bytes)
        .expect("Could not decompress package file");
    let decompressed_package_bytes = &decompressed_package[..];
    let mut extracted_package = tar::Archive::new(decompressed_package_bytes);
    extracted_package
        .unpack(&package_extract_path)
        .expect("Could not unpack package");

    println!(
        "Extracted package file '{}' to filesystem (path: {})",
        package_file_name, package_extract_path
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
    let file = File::create(path).unwrap(); // Create file which we will write to
    let mut buffered_writer = BufWriter::new(file); // Create a buffered writer, allowing us to modify the file we've just created
    buffered_writer
        .write_all(data_to_write)
        .with_context(|| format!("Could not write data to {}", path))
        .unwrap(); // Write data to file
    buffered_writer.flush().unwrap(); // Empty out the data from memory after we've written to the file
}

pub fn add_repository(matches: &clap::ArgMatches) {
    let url = matches
        .value_of("PATH")
        .with_context(|| "No path to a package was given".to_string())
        .unwrap();
    add_repository_with_url(url.to_owned());
}

pub fn add_repository_with_url(url: String) -> Repository {
    let mut repos = load_repositories();
    let client = reqwest::blocking::Client::new();
    let repo = push_repository(&url, &mut repos, &client);
    write_file("/etc/gany/repos.sd", &bincode::serialize(&repos).unwrap());
    repo
}

pub fn push_repository(
    url: &str,
    repos: &mut Vec<Repository>,
    client: &reqwest::blocking::Client,
) -> Repository {
    let response = client
        .get(url)
        .send()
        .unwrap_or_else(|_| panic!("Unable to download repository data from \'{}\' … ", url));
    let repo: Repository = bincode::deserialize(response.text().unwrap().as_bytes()).unwrap();
    repos.push(repo.clone());
    repo
}

pub fn sync_repositories() {
    let client = reqwest::blocking::Client::new();
    let repos = load_repositories();
    let mut new_repos: Vec<Repository> = vec![];
    for repo in &repos {
        push_repository(&repo.address, &mut new_repos, &client);
    }
}

/// Load repository data from filesystem into memory
pub fn load_repositories() -> Vec<Repository> {
    let repositories_file = &fs::read("/etc/gany/repos.sd");
    if let Ok(file) = repositories_file {
        let repositories: Vec<Repository> = bincode::deserialize(file).unwrap();
        repositories
    } else {
        // Make file, then call function again
        let mut repositories: Vec<Repository> = vec![];
        let blank_packages: Vec<Package> = vec![];
        let core_repo = Repository {
            arch: std::env::consts::ARCH.to_string(),
            name: "core".to_string(),
            description: "The core software repository for Cards.".to_string(),
            address: "".to_string(),
            packages: blank_packages,
        };
        repositories.push(core_repo);
        write_file(
            "/etc/gany/repos.sd",
            &bincode::serialize(&repositories).unwrap(),
        );
        load_repositories()
    }
}
