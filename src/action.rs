use std::fmt::Display;
use std::ops::Deref;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

// ---
// --- ActionType
// ---

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
#[derive(PartialEq)]
pub enum ActionType {
    Unknown = 0,
    CopyPath = 100,
    FileTransfer = 101,
    Realise = 102,
    CopyPaths = 103,
    Builds = 104,
    Build = 105,
    OptimiseStore = 106,
    VerifyPaths = 107,
    Substitute = 108,
    QueryPathInfo = 109,
    PostBuildHook = 110,
    BuildWaiting = 111,
}

impl Default for ActionType {
    fn default() -> Self {
        Self::Unknown
    }
}

// ---
// --- BuildStepId
// ---

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Hash)]
pub struct BuildStepId(u64);

impl Deref for BuildStepId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for BuildStepId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BuildStepId> for u64 {
    fn from(val: BuildStepId) -> Self {
        val.0
    }
}

impl Display for BuildStepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---
// --- Action
// ---

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    Msg {
        level: u8, // TODO
        msg: String,
    },
    Start {
        #[serde(rename = "type")]
        action_type: ActionType,
        id: BuildStepId,
        level: u8, // TODO
        parent: BuildStepId,
        text: String,
        #[serde(default)]
        fields: StartFields,
    },
    Result {
        #[serde(rename = "type")]
        action_type: ActionType,
        id: BuildStepId,
        fields: ResultFields,
    },
    Stop {
        id: BuildStepId,
    },
}

/// ---
/// --- Fields
/// ---

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StartFields {
    None,
    Build((String, String, u64, u64)),
    Download([String; 1]),
    Substitute([String; 2]),
    Copy([String; 3]),
}

impl Default for StartFields {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultFields {
    Progress([u64; 4]),
    Whatever([u64; 2]),
    Msg([String; 1]),
}
