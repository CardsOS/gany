use crate::repository::Repository;
use semver::VersionReq;
use std::collections::HashSet;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// A package in a repository
pub struct Package {
    /// The CPU architecture intended for use with the packaged binaries
    pub arch: String,
    /// The name of the package
    pub name: String,
    /// The description of the package
    pub description: String,
    /// The version of the packaged software
    pub version: Version,
    /// The packages that a package depends on
    pub dependencies: Option<HashSet<PackageRequirement>>,
    /// The packages that a package conflicts with
    pub conflicts: Option<HashSet<PackageRequirement>>,
    /// The files that a package owns, including potential ghost files
    pub files: Vec<PathBuf>,
    /// The SHA3-256 hash of the LZ4-compressed archive the software is packaged in
    pub keccak: Option<String>,
}

/// A package requirement which may be fulfilled by a package
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PackageRequirement {
    pub arch: String,
    pub name: String,
    pub version: VersionReq,
    pub dependencies: Option<HashSet<PackageRequirement>>,
    pub conflicts: Option<HashSet<PackageRequirement>>,
}

/// Find candidate packages that fulfill a package requirement
///
/// # Arguments
///
/// * `package_requirement` - A requirement for a package (required)
pub async fn get_candidate_packages(package_requirement: &PackageRequirement) -> HashSet<Package> {
    let mut candidate_packages: HashSet<Package> = HashSet::new();
    let repositories: HashSet<Repository> = repository::fetch_repositories(false).await?;
    for repository in repositories {
        for potential_candidate in repository.packages.unwrap_or_default() {
            if package_requirement.arch == potential_candidate.arch
                && package_requirement.name == potential_candidate.name
                && package_requirement
                    .version
                    .matches(&potential_candidate.version)
                && package_requirement.dependencies == potential_candidate.dependencies
                && package_requirement.conflicts == potential_candidate.conflicts
            {
                candidate_packages.insert(package_candidate);
            }
        }
    }
    candidate_packages
}

/// Create a set of all packages involved in a transaction
///
/// # Arguments
///
/// * `packages` - A set of packages requested in a transaction (required)
pub async fn crawl_package_tree(packages: &HashSet<Package>) -> HashSet<Package> {
    let mut all_packages_set: HashSet<Package> = HashSet::new();
    for package in packages {
        // Skip packages that do not support the user's architecture
        if package.arch != *ARCH {
            continue;
        }
        all_packages_set.insert(package.clone());
        // Determine dependencies and conflicts for each package
        let dependencies_and_conflicts: HashSet<PackageRequirement> = &package
            .dependencies
            .unwrap_or_default()
            .union(&package.conflicts.unwrap_or_default())
            .collect();
        for package_requirement in dependencies_and_conflicts {
            let candidate_packages = get_candidate_packages(&package_requirement).await?;
            // Recursively crawl the package tree
            for candidate_package in candidate_packages {
                all_packages_set = all_packages_set
                    .union(build_package_tree(&candidate_packages))
                    .collect();
            }
        }
    }
    all_packages_set
}

/// Determines how a set of packages can be installed without conflicts, if possible.
///
/// # Arguments
///
/// * `packages` - A set of packages requested for installation (required)
pub async fn solve_packages(packages: &HashSet<&Package>) -> Result<Vec<Package>> {
    let mut solver = Solver::new();
    let mut transaction_formula = CnfFormula::new();
    let all_packages_set = crawl_package_tree(packages).await?;

    // Create a map of variables by package to ensure that each package has a unique variable in the formula
    let mut all_packages_map: HashMap<Package, Var> = HashMap::new();
    for package in all_packages_set {
        all_packages_map.insert(package, solver.new_var());
    }
    for package in packages {
        let package_var = all_packages_map.get(package).unwrap();
        for dependency in package.dependencies.unwrap_or_default() {
            let dependency_candidates = get_candidate_packages(&dependency).await?;
            let dependency_candidate_vars: Vec<Var> = dependency_candidates
                .into_iter()
                .map(|candidate| all_packages_map.get(&candidate).unwrap())
                .collect();
            transaction_formula.add_clause(&[[!package_var], &dependency_candidate_vars].concat());
            // Package implies dependency
        }
        for conflict in package.conflicts.unwrap_or_default() {
            let conflict_candidates = get_candidate_packages(&conflict).await?;
            let conflict_candidate_vars: Vec<Var> = conflict_candidates
                .into_iter()
                .map(|candidate| all_packages_map.get(&candidate).unwrap())
                .collect();
            let conflict_candidate_lits: Vec<Lit> = conflict_candidate_vars
                .into_iter()
                .map(|var| !var)
                .collect();
            transaction_formula.add_clause(&[[!package_var], &conflict_candidate_lits].concat());
            // Both package and conflict cannot be installed
        }
    }
    solver.add_formula(&transaction_formula);
    if solver.solve() {
        let mut transaction: Vec<Package> = Vec::new();
        let solution_option = solver.model();
        if let Some(solution) = solution_option {
            for literal in solution {
                if literal.is_pos() {
                    let package = all_packages_map
                        .iter()
                        .find(|(_, var)| **var == literal.var())
                        .unwrap()
                        .0;
                    transaction.push(package.clone());
                }
            }
        }
        Ok(transaction)
    } else {
        Err(PackageInstallationError::UnableToSolveTransaction)
    }
}
