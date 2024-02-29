use miette::Diagnostic;
use thiserror::Error;
use url::Url;

#[derive(Error, Diagnostic, Debug)]
pub enum FetchRepositoriesError {
    #[error("Unable to synchronise repository data as the repository list is unable to be read.")]
    #[diagnostic(code(fetch_repositories::unable_to_read_repository_list))]
    UnableToReadRepositoryList,
    #[error("Unable to synchronise repository data as the repository list is unable to be deserialised.")]
    #[diagnostic(code(fetch_repositories::unable_to_deserialise_repository_list))]
    UnableToDeserialiseRepositoryList,
    #[error("Unable to download repository data from {0}.")]
    #[diagnostic(code(fetch_repositories::unable_to_download_repository_data))]
    UnableToDownloadRepositoryData(Url),
    #[error("Unable to read repository data from the filesystem.")]
    #[diagnostic(code(fetch_repositories::unable_to_read_repository_data))]
    UnableToReadRepositoryData,
    #[error("Unable to deserialise repository data from the filesystem.")]
    #[diagnostic(code(fetch_repositories::unable_to_deserialise_repository_data_filesystem))]
    UnableToDeserialiseRepositoryDataFilesystem,
    #[error("Unable to deserialise repository data from {0}.")]
    #[diagnostic(code(fetch_repositories::unable_to_deserialise_repository_data_internet))]
    UnableToDeserialiseRepositoryDataInternet(Url),
}

#[derive(Error, Diagnostic, Debug)]
pub enum PackageInstallationError {
    #[error("Unable to install requested packages without a conflict.")]
    #[diagnostic(code(package_installation::unable_to_solve_transaction))]
    UnableToSolveTransaction,
}
