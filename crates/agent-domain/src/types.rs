//! Strong domain types that replace meaning-bearing raw primitives.
//!
//! These value objects move critical invariants to construction time. Instead
//! of asking every caller to remember that repository paths must stay relative
//! or that mode names must be lowercase slugs, Ferrify encodes those rules in
//! dedicated types and rejects invalid values at the boundary.

use std::{
    borrow::Borrow,
    fmt,
    path::{Component, Path},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validation failures for domain value objects.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainTypeError {
    /// The provided repository path was empty.
    #[error("repository path must not be empty")]
    EmptyRepoPath,
    /// The provided repository path was absolute.
    #[error("repository path `{value}` must remain relative to the workspace")]
    AbsoluteRepoPath {
        /// The rejected path value.
        value: String,
    },
    /// The provided repository path attempted to escape the workspace root.
    #[error("repository path `{value}` must not contain parent-directory traversal")]
    TraversingRepoPath {
        /// The rejected path value.
        value: String,
    },
    /// The provided slug was empty.
    #[error("{kind} must not be empty")]
    EmptySlug {
        /// The semantic kind of slug that failed validation.
        kind: &'static str,
    },
    /// The provided slug contained unsupported characters.
    #[error("{kind} `{value}` may contain only lowercase ascii letters, digits, `-`, and `_`")]
    InvalidSlug {
        /// The semantic kind of slug that failed validation.
        kind: &'static str,
        /// The rejected slug value.
        value: String,
    },
}

/// A repository-relative path that cannot escape the workspace root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(String);

impl RepoPath {
    /// Creates a validated repository-relative path.
    ///
    /// `RepoPath` rejects empty values, absolute paths, and any path that tries
    /// to escape the workspace with `..`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainTypeError`] when the value is empty, absolute, or
    /// contains parent-directory traversal.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_domain::RepoPath;
    ///
    /// # fn main() -> Result<(), agent_domain::DomainTypeError> {
    /// let path = RepoPath::new("crates/agent-cli/src/main.rs")?;
    /// assert_eq!(path.as_str(), "crates/agent-cli/src/main.rs");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, DomainTypeError> {
        let value = value.into();
        if value.is_empty() {
            return Err(DomainTypeError::EmptyRepoPath);
        }

        let path = Path::new(&value);
        if path.is_absolute() {
            return Err(DomainTypeError::AbsoluteRepoPath { value });
        }

        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(DomainTypeError::TraversingRepoPath { value });
        }

        Ok(Self(value))
    }

    /// Returns the path as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the value object and returns the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for RepoPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for RepoPath {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RepoPath {
    type Err = DomainTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RepoPath {
    type Error = DomainTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for RepoPath {
    type Error = DomainTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.into_inner()
    }
}

/// A stable slug identifying an execution mode.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModeSlug(String);

impl ModeSlug {
    /// Creates a validated mode slug.
    ///
    /// Mode slugs are stable identifiers used in policy files and orchestration
    /// logic. They are intentionally restricted to lowercase ASCII letters,
    /// digits, `-`, and `_`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainTypeError`] when the value is empty or contains
    /// unsupported characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_domain::ModeSlug;
    ///
    /// # fn main() -> Result<(), agent_domain::DomainTypeError> {
    /// let slug = ModeSlug::new("implementer")?;
    /// assert_eq!(slug.as_str(), "implementer");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, DomainTypeError> {
        let value = value.into();
        validate_slug("mode slug", &value)?;
        Ok(Self(value))
    }

    /// Returns the slug as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the slug and returns the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for ModeSlug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for ModeSlug {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ModeSlug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ModeSlug {
    type Err = DomainTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for ModeSlug {
    type Error = DomainTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for ModeSlug {
    type Error = DomainTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ModeSlug> for String {
    fn from(value: ModeSlug) -> Self {
        value.into_inner()
    }
}

/// A stable slug identifying an approval profile.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ApprovalProfileSlug(String);

impl ApprovalProfileSlug {
    /// Creates a validated approval-profile slug.
    ///
    /// Approval-profile slugs identify policy bundles under
    /// `.agent/approvals/*.yaml`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainTypeError`] when the value is empty or contains
    /// unsupported characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_domain::ApprovalProfileSlug;
    ///
    /// # fn main() -> Result<(), agent_domain::DomainTypeError> {
    /// let slug = ApprovalProfileSlug::new("default")?;
    /// assert_eq!(slug.as_str(), "default");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, DomainTypeError> {
        let value = value.into();
        validate_slug("approval profile slug", &value)?;
        Ok(Self(value))
    }

    /// Returns the slug as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the slug and returns the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for ApprovalProfileSlug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for ApprovalProfileSlug {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ApprovalProfileSlug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ApprovalProfileSlug {
    type Err = DomainTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for ApprovalProfileSlug {
    type Error = DomainTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for ApprovalProfileSlug {
    type Error = DomainTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ApprovalProfileSlug> for String {
    fn from(value: ApprovalProfileSlug) -> Self {
        value.into_inner()
    }
}

fn validate_slug(kind: &'static str, value: &str) -> Result<(), DomainTypeError> {
    if value.is_empty() {
        return Err(DomainTypeError::EmptySlug { kind });
    }

    if value.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '-' | '_')
    }) {
        Ok(())
    } else {
        Err(DomainTypeError::InvalidSlug {
            kind,
            value: value.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{ApprovalProfileSlug, DomainTypeError, ModeSlug, RepoPath};

    #[test]
    fn repo_path_rejects_workspace_escape() {
        assert_eq!(
            RepoPath::new("../secrets.txt"),
            Err(DomainTypeError::TraversingRepoPath {
                value: "../secrets.txt".to_owned(),
            })
        );
    }

    #[test]
    fn mode_slug_rejects_uppercase_characters() {
        assert_eq!(
            ModeSlug::new("Architect"),
            Err(DomainTypeError::InvalidSlug {
                kind: "mode slug",
                value: "Architect".to_owned(),
            })
        );
    }

    #[test]
    fn approval_profile_slug_roundtrips_through_serde() {
        let slug = ApprovalProfileSlug::new("default_profile")
            .unwrap_or_else(|error| panic!("default_profile should be valid: {error}"));
        let encoded = to_string(&slug)
            .unwrap_or_else(|error| panic!("approval profile slug should serialize: {error}"));
        let decoded: ApprovalProfileSlug = from_str(&encoded)
            .unwrap_or_else(|error| panic!("approval profile slug should deserialize: {error}"));

        assert_eq!(decoded, slug);
    }
}
