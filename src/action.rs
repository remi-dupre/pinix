use std::fmt::Display;
use std::ops::Deref;

use serde::de;
use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;

// ---
// --- ActionType
// ---

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
#[derive(Eq, PartialEq)]
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
        level: u8,
        msg: String,
    },
    Start {
        #[serde(rename = "type")]
        action_type: ActionType,
        id: BuildStepId,
        level: u8,
        parent: BuildStepId,
        text: String,
        #[serde(default)]
        fields: StartFields,
    },
    Result(ActionResult),
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

#[derive(Debug)]
pub enum ResultFields {
    BuildLogLine(String),
    SetPhase(String),
    Progress {
        done: u64,
        expected: u64,
        running: u64,
        failed: u64,
    },
    SetExpected {
        action: ActionType,
        expected: u64,
    },
}

#[derive(Debug)]
pub struct ActionResult {
    pub id: BuildStepId,
    pub fields: ResultFields,
}

impl<'de> Deserialize<'de> for ActionResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct RawActionResult {
            id: BuildStepId,
            #[serde(rename = "type")]
            result_type: u8,
            fields: serde_json::Value,
        }

        let raw = RawActionResult::deserialize(deserializer)?;

        let fields = match raw.result_type {
            100 => todo!("FileLinked({})", raw.fields),
            101 => {
                let [line] = serde_json::from_value(raw.fields).map_err(|err| {
                    de::Error::invalid_value(
                        de::Unexpected::Other(&err.to_string()),
                        &"an array with a single string",
                    )
                })?;

                ResultFields::BuildLogLine(line)
            }
            102 => todo!("UntrustedPath({})", raw.fields),
            103 => todo!("CorruptedPath({})", raw.fields),
            104 => {
                let [phase] = serde_json::from_value(raw.fields).map_err(|err| {
                    de::Error::invalid_value(
                        de::Unexpected::Other(&err.to_string()),
                        &"an array with a single string",
                    )
                })?;

                ResultFields::SetPhase(phase)
            }
            105 => {
                let (done, expected, running, failed) = serde_json::from_value(raw.fields)
                    .map_err(|err| {
                        de::Error::invalid_value(
                            de::Unexpected::Other(&err.to_string()),
                            &"an array with 4 integers",
                        )
                    })?;

                ResultFields::Progress {
                    done,
                    expected,
                    running,
                    failed,
                }
            }
            106 => {
                let (action, expected) = serde_json::from_value(raw.fields).map_err(|err| {
                    de::Error::invalid_value(
                        de::Unexpected::Other(&err.to_string()),
                        &"an array with an action type and an expected value",
                    )
                })?;

                ResultFields::SetExpected { action, expected }
            }
            107 => todo!("PostBuildLogLine({})", raw.fields),
            v => {
                return Err(de::Error::invalid_value(
                    de::Unexpected::Unsigned(v.into()),
                    &"a result type ID, from 100 to 107",
                ))
            }
        };

        Ok(ActionResult { id: raw.id, fields })
    }
}
