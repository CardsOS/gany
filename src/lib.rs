use anyhow::Context;
use std::io::Write;
use std::io::BufWriter;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::env;

/// Add a package to your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Path to a Mokk (required)
pub fn add_package(matches: &clap::ArgMatches)
{
    let package_name = matches
    .value_of("PACKAGE_NAME")
    .with_context(|| "No path to a Mokk was given".to_string())
    .unwrap();
env::set_current_dir(package_name)
    .with_context(|| format!("Could not read a Mokk at {}", package_name))
    .unwrap();
}

/// Drop a package from your software installation
///
/// # Arguments
///
/// * `PACKAGE_NAME` - Path to a Mokk (required)
pub fn drop_package(matches: &clap::ArgMatches)
{
    let package_name = matches
    .value_of("PACKAGE_NAME")
    .with_context(|| "No path to a Mokk was given".to_string())
    .unwrap();
env::set_current_dir(package_name)
    .with_context(|| format!("Could not read a Mokk at {}", package_name))
    .unwrap();
}

/// Refresh the local package repository with one from a remote software distribution
pub fn refresh_remote_packages() {

}

/// Upgrade your local packages with newer versions held in a remote software distribution
pub fn upgrade_packages() {

}

/// Package a piece of software for future distribution
///
/// # Arguments
///
/// * `PATH` - Path to a Mokk (required)
pub fn create_package(matches: &clap::ArgMatches)
{
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
    buffered_writer.write_all(data_to_write).with_context(|| format!("Could not write data to {}", path)).unwrap(); // Write data to file
    buffered_writer.flush().unwrap(); // Empty out the data from memory after we've written to the file
}