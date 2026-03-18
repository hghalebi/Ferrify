use serde::{Deserialize, Serialize};

use crate::TrustLevel;

/// The operational role an input plays in Ferrify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputRole {
    /// The direct task intent from the operator.
    Goal,
    /// Repository or platform policy that can constrain execution.
    Policy,
    /// Source code or manifests from the workspace.
    Code,
    /// Verification output and other execution evidence.
    Evidence,
    /// Text from tools or external sources that must not change authority.
    UntrustedText,
}

impl InputRole {
    /// Returns whether this role is allowed to influence authority decisions.
    #[must_use]
    pub fn can_define_authority(self) -> bool {
        matches!(self, Self::Goal | Self::Policy)
    }
}

/// A lightweight provenance record for an input observed during a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedInput {
    /// The operational role attached to the input.
    pub role: InputRole,
    /// Where the input came from.
    pub source: String,
    /// A concise summary of how the input is used.
    pub summary: String,
    /// The trust level attached to the input source.
    pub trust_level: TrustLevel,
}

#[cfg(test)]
mod tests {
    use super::InputRole;

    #[test]
    fn only_goal_and_policy_define_authority() {
        assert!(InputRole::Goal.can_define_authority());
        assert!(InputRole::Policy.can_define_authority());
        assert!(!InputRole::Code.can_define_authority());
        assert!(!InputRole::Evidence.can_define_authority());
        assert!(!InputRole::UntrustedText.can_define_authority());
    }
}
