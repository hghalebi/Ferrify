//! Repository modeling and working-context selection.
//!
//! `ferrify-context` is the read-only exploration layer in Ferrify. It inspects a
//! repository, records structural facts such as workspace members and toolchain
//! files, and produces a bounded working set that later stages can use without
//! carrying the entire repo into memory.
//!
//! The crate follows a structural-first read order. It looks at root manifests,
//! toolchain configuration, CI files, repository policy, and only then expands
//! into nearby code. That order matters because Ferrify treats current
//! repository evidence as stronger than remembered conventions.
//!
//! # Examples
//!
//! ```no_run
//! use ferrify_context::RepoModeler;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let model = RepoModeler::scan(Path::new("."))?;
//! assert!(!model.crates.is_empty());
//! # Ok(())
//! # }
//! ```

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use agent_domain::{DependencyPolicy, DomainTypeError, RepoPath, TrustLevel};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

/// The broad workspace shape discovered during scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceKind {
    /// A single package rooted at the repository root.
    SingleCrate,
    /// A Cargo workspace with one or more member crates.
    MultiCrate,
}

/// Facts collected for one Cargo crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateFacts {
    /// The crate name from `Cargo.toml`.
    pub name: String,
    /// The manifest path for the crate.
    pub manifest_path: RepoPath,
    /// The Rust edition declared by the crate.
    pub edition: String,
    /// Dependency names observed in the manifest.
    pub dependencies: BTreeSet<String>,
    /// Source files that anchor the crate in the working set.
    pub source_files: Vec<RepoPath>,
}

/// Toolchain files and CI entry points discovered at the root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ToolchainFacts {
    /// `rust-toolchain.toml` when present.
    pub rust_toolchain_path: Option<RepoPath>,
    /// `.cargo/config.toml` when present.
    pub cargo_config_path: Option<RepoPath>,
    /// CI workflow files under `.github/workflows`.
    pub ci_workflows: Vec<RepoPath>,
}

/// The async runtime posture inferred from dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsyncModel {
    /// The repo uses Tokio.
    Tokio,
    /// The repo uses async-std.
    AsyncStd,
    /// The repo does not appear to use an async runtime.
    NoneKnown,
    /// The runtime is not obvious yet.
    Unknown,
}

/// The error handling style inferred from dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorStyle {
    /// The repo prefers `thiserror`.
    ThisError,
    /// The repo prefers `anyhow`.
    Anyhow,
    /// The repo uses hand-rolled error types or standard errors.
    Standard,
    /// The style has not been established yet.
    Unknown,
}

/// The logging style inferred from dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoggingStyle {
    /// The repo uses `tracing`.
    Tracing,
    /// The repo uses the `log` facade.
    Log,
    /// The repo does not appear to use logging crates.
    NoneKnown,
    /// The style has not been established yet.
    Unknown,
}

/// The testing style inferred from dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStyle {
    /// The repo uses `trycmd`.
    Trycmd,
    /// The repo uses `assert_cmd`.
    AssertCmd,
    /// The repo relies on standard Rust tests.
    Standard,
    /// The style has not been established yet.
    Unknown,
}

/// The CLI implementation style inferred from dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliStyle {
    /// The repo uses `clap`.
    Clap,
    /// The repo uses `pico-args`.
    PicoArgs,
    /// The repo does not appear to define a CLI yet.
    NoneKnown,
    /// The style has not been established yet.
    Unknown,
}

/// A discovered public API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiBoundary {
    /// The crate that owns the boundary.
    pub crate_name: String,
    /// Paths that define the public boundary.
    pub public_paths: Vec<RepoPath>,
}

/// A repository fact preserved across compaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFact {
    /// The fact subject.
    pub subject: String,
    /// The fact detail.
    pub detail: String,
    /// The trust classification for the fact.
    pub trust_level: TrustLevel,
}

/// An unresolved question that should remain visible to later stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenQuestion {
    /// The question text.
    pub question: String,
}

/// The compact working set handed to planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingSet {
    /// Files selected for the active context.
    pub files: Vec<RepoPath>,
    /// Symbol-like identifiers carried into planning.
    pub symbols: Vec<String>,
    /// Durable facts extracted from the repository.
    pub facts: Vec<RepoFact>,
    /// Open questions that still matter.
    pub open_questions: Vec<OpenQuestion>,
}

/// Limits for context selection before compaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    /// Maximum number of files to retain.
    pub max_files: u16,
    /// Maximum number of lines to retain.
    pub max_lines: u32,
    /// Maximum number of tool results to retain.
    pub max_tool_results: u16,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_files: 16,
            max_lines: 800,
            max_tool_results: 12,
        }
    }
}

/// The compacted context snapshot that survives between stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Facts preserved from earlier exploration.
    pub preserved_facts: Vec<RepoFact>,
    /// The active plan summary.
    pub current_plan: String,
    /// Active failures that need follow-up.
    pub active_failures: Vec<String>,
}

/// The repository model built before planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoModel {
    /// Whether the repository is a single crate or a workspace.
    pub workspace_kind: WorkspaceKind,
    /// Facts for discovered crates.
    pub crates: Vec<CrateFacts>,
    /// The default edition inferred from the root or first crate.
    pub edition: String,
    /// Toolchain and CI facts discovered at the root.
    pub toolchain: ToolchainFacts,
    /// The inferred async runtime posture.
    pub async_model: AsyncModel,
    /// The inferred error handling style.
    pub error_style: ErrorStyle,
    /// The inferred logging style.
    pub logging_style: LoggingStyle,
    /// The inferred test style.
    pub test_style: TestStyle,
    /// The inferred CLI style.
    pub cli_style: CliStyle,
    /// The repository stance on dependency changes.
    pub dependency_policy: DependencyPolicy,
    /// Public API boundaries that should constrain planning.
    pub public_api_boundaries: Vec<ApiBoundary>,
    /// Files read in the prescribed discovery order.
    pub read_order: Vec<RepoPath>,
}

/// Scans a repository root and builds a `RepoModel`.
#[derive(Debug, Default)]
pub struct RepoModeler;

impl RepoModeler {
    /// Scans the repository root using Ferrify's structural-first read order.
    ///
    /// The scan prefers root manifests, toolchain files, CI entry points, and
    /// repository policy before it expands into crate-specific source files.
    /// That ordering keeps planning grounded in the repo's declared structure.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError`] when a required manifest cannot be read or
    /// parsed, when repository-relative paths cannot be normalized into
    /// [`RepoPath`], or when the filesystem cannot be traversed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ferrify_context::RepoModeler;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo_model = RepoModeler::scan(Path::new("."))?;
    /// println!("discovered {} crate(s)", repo_model.crates.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan(root: &Path) -> Result<RepoModel, ContextError> {
        let root_manifest = root.join("Cargo.toml");
        let mut read_order = Vec::new();
        push_if_exists(&mut read_order, root, &root_manifest)?;
        push_if_exists(&mut read_order, root, &root.join("rust-toolchain.toml"))?;
        push_if_exists(
            &mut read_order,
            root,
            &root.join(".cargo").join("config.toml"),
        )?;

        let workflow_dir = root.join(".github").join("workflows");
        if workflow_dir.is_dir() {
            let mut workflow_paths = fs::read_dir(&workflow_dir)?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            workflow_paths.sort();
            for path in workflow_paths {
                push_if_exists(&mut read_order, root, &path)?;
            }
        }

        push_if_exists(&mut read_order, root, &root.join("AGENTS.md"))?;
        for directory in ["rules", "path-rules", "modes", "approvals"] {
            push_directory_entries_if_exists(
                &mut read_order,
                root,
                &root.join(".agent").join(directory),
            )?;
        }
        for candidate in ["README.md", "README"] {
            push_if_exists(&mut read_order, root, &root.join(candidate))?;
        }

        let root_value = parse_manifest(&root_manifest)?;
        let member_manifest_paths = member_manifests(root, &root_value);
        let mut crates = Vec::new();
        for manifest in &member_manifest_paths {
            push_if_exists(&mut read_order, root, manifest)?;
            crates.push(scan_crate(root, manifest)?);
        }

        let all_dependencies = crates
            .iter()
            .flat_map(|facts| facts.dependencies.iter().cloned())
            .collect::<BTreeSet<_>>();

        let workspace_kind = if crates.len() > 1 {
            WorkspaceKind::MultiCrate
        } else {
            WorkspaceKind::SingleCrate
        };

        let toolchain = ToolchainFacts {
            rust_toolchain_path: relative_path(root, &root.join("rust-toolchain.toml"))?,
            cargo_config_path: relative_path(root, &root.join(".cargo").join("config.toml"))?,
            ci_workflows: read_order
                .iter()
                .filter(|path| path.as_str().starts_with(".github/workflows/"))
                .cloned()
                .collect(),
        };

        let public_api_boundaries = crates
            .iter()
            .filter(|facts| {
                facts
                    .source_files
                    .iter()
                    .any(|path| path.as_str().ends_with("/src/lib.rs"))
            })
            .map(|facts| ApiBoundary {
                crate_name: facts.name.clone(),
                public_paths: facts
                    .source_files
                    .iter()
                    .filter(|path| path.as_str().ends_with("/src/lib.rs"))
                    .cloned()
                    .collect(),
            })
            .collect();

        let edition = crates
            .first()
            .map(|facts| facts.edition.clone())
            .unwrap_or_else(|| "2024".to_owned());

        Ok(RepoModel {
            workspace_kind,
            crates,
            edition,
            toolchain,
            async_model: infer_async_model(&all_dependencies),
            error_style: infer_error_style(&all_dependencies),
            logging_style: infer_logging_style(&all_dependencies),
            test_style: infer_test_style(&all_dependencies),
            cli_style: infer_cli_style(&all_dependencies),
            dependency_policy: DependencyPolicy::AllowApproved,
            public_api_boundaries,
            read_order,
        })
    }
}

/// Builds a compact working set from a `RepoModel`.
#[derive(Debug, Default)]
pub struct ContextBuilder;

impl ContextBuilder {
    /// Selects a bounded working set from the repository model.
    #[must_use]
    pub fn build(repo_model: &RepoModel, budget: ContextBudget) -> WorkingSet {
        let mut files = repo_model.read_order.clone();
        for facts in &repo_model.crates {
            for source_file in &facts.source_files {
                if !files.contains(source_file) {
                    files.push(source_file.clone());
                }
            }
        }
        files.truncate(usize::from(budget.max_files));

        let facts = vec![
            RepoFact {
                subject: "workspace_kind".to_owned(),
                detail: format!("{:?}", repo_model.workspace_kind),
                trust_level: TrustLevel::RepoCode,
            },
            RepoFact {
                subject: "crate_count".to_owned(),
                detail: repo_model.crates.len().to_string(),
                trust_level: TrustLevel::RepoCode,
            },
            RepoFact {
                subject: "cli_style".to_owned(),
                detail: format!("{:?}", repo_model.cli_style),
                trust_level: TrustLevel::RepoCode,
            },
        ];

        let open_questions = if repo_model.public_api_boundaries.is_empty() {
            vec![OpenQuestion {
                question: "No library boundary was inferred; public API impact is an inference."
                    .to_owned(),
            }]
        } else {
            Vec::new()
        };

        WorkingSet {
            files,
            symbols: repo_model
                .crates
                .iter()
                .map(|facts| facts.name.clone())
                .collect(),
            facts,
            open_questions,
        }
    }

    /// Compacts the current state into a durable snapshot.
    #[must_use]
    pub fn snapshot(
        working_set: &WorkingSet,
        current_plan: impl Into<String>,
        active_failures: Vec<String>,
    ) -> ContextSnapshot {
        ContextSnapshot {
            preserved_facts: working_set.facts.clone(),
            current_plan: current_plan.into(),
            active_failures,
        }
    }
}

/// Errors produced while scanning repository context.
#[derive(Debug, Error)]
pub enum ContextError {
    /// Filesystem access failed.
    #[error("failed to read repository context: {0}")]
    Io(#[from] std::io::Error),
    /// Manifest parsing failed.
    #[error("failed to parse Cargo manifest {path}: {source}")]
    Manifest {
        /// The manifest path that failed to parse.
        path: PathBuf,
        /// The underlying parse error.
        source: toml::de::Error,
    },
    /// A discovered repository path violated the domain path invariants.
    #[error("failed to validate repository path: {0}")]
    InvalidRepoPath(#[from] DomainTypeError),
    /// A discovered file was not rooted under the scanned workspace.
    #[error("path `{0}` is outside the scanned workspace root")]
    ExternalWorkspacePath(PathBuf),
}

fn push_if_exists(
    read_order: &mut Vec<RepoPath>,
    root: &Path,
    candidate: &Path,
) -> Result<(), ContextError> {
    if candidate.exists()
        && let Some(relative) = relative_path(root, candidate)?
    {
        read_order.push(relative);
    }
    Ok(())
}

fn push_directory_entries_if_exists(
    read_order: &mut Vec<RepoPath>,
    root: &Path,
    directory: &Path,
) -> Result<(), ContextError> {
    if !directory.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();

    for entry in entries {
        push_if_exists(read_order, root, &entry)?;
    }

    Ok(())
}

fn parse_manifest(path: &Path) -> Result<Value, ContextError> {
    let raw = fs::read_to_string(path)?;
    toml::from_str(&raw).map_err(|source| ContextError::Manifest {
        path: path.to_path_buf(),
        source,
    })
}

fn member_manifests(root: &Path, manifest: &Value) -> Vec<PathBuf> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(Value::as_str)
                .map(|member| root.join(member).join("Cargo.toml"))
                .collect()
        })
        .unwrap_or_else(|| vec![root.join("Cargo.toml")])
}

fn scan_crate(root: &Path, manifest_path: &Path) -> Result<CrateFacts, ContextError> {
    let manifest = parse_manifest(manifest_path)?;
    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .cloned()
        .unwrap_or_default();
    let dependencies = dependency_names(&manifest);
    let crate_root = manifest_path.parent().unwrap_or(root).to_path_buf();

    let source_files = ["src/lib.rs", "src/main.rs"]
        .into_iter()
        .map(|relative| crate_root.join(relative))
        .filter_map(|path| relative_path(root, &path).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CrateFacts {
        name: package
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned(),
        manifest_path: relative_path(root, manifest_path)?
            .ok_or_else(|| ContextError::ExternalWorkspacePath(manifest_path.to_path_buf()))?,
        edition: package
            .get("edition")
            .and_then(Value::as_str)
            .unwrap_or("2024")
            .to_owned(),
        dependencies,
        source_files,
    })
}

fn dependency_names(manifest: &Value) -> BTreeSet<String> {
    manifest
        .get("dependencies")
        .and_then(Value::as_table)
        .map(|dependencies| dependencies.keys().cloned().collect())
        .unwrap_or_default()
}

fn infer_async_model(dependencies: &BTreeSet<String>) -> AsyncModel {
    if dependencies.contains("tokio") {
        AsyncModel::Tokio
    } else if dependencies.contains("async-std") {
        AsyncModel::AsyncStd
    } else if dependencies.is_empty() {
        AsyncModel::Unknown
    } else {
        AsyncModel::NoneKnown
    }
}

fn infer_error_style(dependencies: &BTreeSet<String>) -> ErrorStyle {
    if dependencies.contains("thiserror") {
        ErrorStyle::ThisError
    } else if dependencies.contains("anyhow") {
        ErrorStyle::Anyhow
    } else if dependencies.is_empty() {
        ErrorStyle::Unknown
    } else {
        ErrorStyle::Standard
    }
}

fn infer_logging_style(dependencies: &BTreeSet<String>) -> LoggingStyle {
    if dependencies.contains("tracing") {
        LoggingStyle::Tracing
    } else if dependencies.contains("log") {
        LoggingStyle::Log
    } else if dependencies.is_empty() {
        LoggingStyle::Unknown
    } else {
        LoggingStyle::NoneKnown
    }
}

fn infer_test_style(dependencies: &BTreeSet<String>) -> TestStyle {
    if dependencies.contains("trycmd") {
        TestStyle::Trycmd
    } else if dependencies.contains("assert_cmd") {
        TestStyle::AssertCmd
    } else if dependencies.is_empty() {
        TestStyle::Unknown
    } else {
        TestStyle::Standard
    }
}

fn infer_cli_style(dependencies: &BTreeSet<String>) -> CliStyle {
    if dependencies.contains("clap") {
        CliStyle::Clap
    } else if dependencies.contains("pico-args") {
        CliStyle::PicoArgs
    } else if dependencies.is_empty() {
        CliStyle::Unknown
    } else {
        CliStyle::NoneKnown
    }
}

fn relative_path(root: &Path, candidate: &Path) -> Result<Option<RepoPath>, ContextError> {
    if candidate.exists() {
        let relative = candidate
            .strip_prefix(root)
            .map_err(|_| ContextError::ExternalWorkspacePath(candidate.to_path_buf()))?
            .display()
            .to_string();
        Ok(Some(RepoPath::new(relative)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use agent_domain::RepoPath;
    use tempfile::tempdir;

    use super::{RepoModeler, WorkspaceKind};

    fn repo_path(value: &str) -> RepoPath {
        match RepoPath::new(value) {
            Ok(path) => path,
            Err(error) => panic!("repo path should be valid in test: {error}"),
        }
    }

    #[test]
    fn repo_modeler_discovers_workspace_members() {
        let tempdir = tempdir().expect("tempdir should be created for context test");
        let root = tempdir.path();

        fs::create_dir_all(root.join("crates").join("app").join("src"))
            .expect("crate source directory should be created for context test");
        fs::create_dir_all(root.join(".agent").join("modes"))
            .expect("mode directory should be created for context test");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/app\"]\n",
        )
        .expect("workspace manifest should be written for context test");
        fs::write(
            root.join("crates").join("app").join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nclap = \"4\"\n",
        )
        .expect("crate manifest should be written for context test");
        fs::write(
            root.join("crates").join("app").join("src").join("main.rs"),
            "fn main() {}\n",
        )
        .expect("crate source should be written for context test");
        fs::write(root.join("AGENTS.md"), "# Rules\n")
            .expect("agents contract should be written for context test");
        fs::write(
            root.join(".agent").join("modes").join("architect.yaml"),
            "slug: architect\npurpose: read only\n",
        )
        .expect("mode file should be written for context test");

        let model = RepoModeler::scan(root).expect("repo model should scan");
        assert_eq!(model.workspace_kind, WorkspaceKind::SingleCrate);
        assert_eq!(model.crates.len(), 1);
        assert_eq!(model.crates[0].name, "app");
        assert!(model.read_order.contains(&repo_path("AGENTS.md")));
        assert!(
            model
                .read_order
                .contains(&repo_path(".agent/modes/architect.yaml"))
        );
    }
}
