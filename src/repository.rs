use crate::package::Package;
use std::collections::HashSet;
use url::Url;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// The data associated with a repository
pub struct Repository {
    /// The name of the repository
    pub name: String,
    /// The description of the repository
    pub description: String,
    /// The address (URL or IP) of the repository
    pub address: String,
    /// The packages within this repository
    pub packages: Option<HashSet<Package>>,
}

/// Fetches repository data from the filesystem or the Internet
pub async fn fetch_repositories(sync: bool) -> Result<HashSet<Repository>> {
    match sync {
        true => sync_repositories().await,
        false => read_repositories(),
    }
}

/// Reads the previously synchronised repositories from the filesystem
pub fn read_repositories() -> Result<HashSet<Repository>> {
    let repositories_file = &fs::read("/etc/gany/gany-repos.bin");
    if let Ok(repositories) = repositories_file {
        let repositories_data = bincode::deserialize(repositories);
        if let Ok(repositories) = repositories_data {
            Ok(repositories)
        } else {
            Err(FetchRepositoriesError::UnableToDeserialiseRepositoryDataFilesystem)
        }
    } else {
        Err(FetchRepositoriesError::UnableToReadRepositoryData)
    }
}

/// Synchronises local repository data with remote sources
pub async fn sync_repositories() -> Result<HashSet<Repository>> {
    let repositories_list_file = &fs::read("/etc/gany/gany-repos.yaml");

    if (sync && repositories_list_file.is_err()) {
        Err(FetchRepositoriesError::UnableToReadRepositoryList)
    }

    if let Ok(repositories_list) = repositories_list_file {
        let repositories_list_yaml: Result<HashSet<Url>, _> =
            serde_yaml::from_slice(repositories_list);
        if let Ok(repositories_list) = repositories_list_yaml {
            let mut new_repositories: HashSet<Repository> = HashSet::new();
            for repository_url in &repositories_list {
                let client = reqwest::Client::new();
                let http_request = client.get(&repository_url.as_str()).send().await;
                if let Ok(response) = http_request {
                    let repository_data = response.bytes().await?;
                    let repository_bin: Result<Repository, _> =
                        bincode::deserialize(&repository_data);
                    if let Ok(repository) = repository_bin {
                        new_repositories.insert(repository);
                    } else {
                        return Err(
                            FetchRepositoriesError::UnableToDeserialiseRepositoryDataInternet(
                                repository_url,
                            ),
                        );
                    }
                } else {
                    return Err(FetchRepositoriesError::UnableToDownloadRepositoryData(
                        repository.address,
                    ));
                }
            }
            write_file(
                "/etc/gany/gany-repos.bin",
                &bincode::serialize(&new_repositories).unwrap(),
            );
            Ok(new_repositories)
        } else {
            Err(FetchRepositoriesError::UnableToDeserialiseRepositoryList)
        }
    } else {
        Err(FetchRepositoriesError::UnableToReadRepositoryList)
    }
}
