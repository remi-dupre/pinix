use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;

use anyhow::Context;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::parse::action_raw::RawAction;

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

#[derive(Debug)]
pub enum StartFields<'a> {
    Unknown,
    CopyPath {
        path: Cow<'a, str>,
        origin: Cow<'a, str>,
        destination: Cow<'a, str>,
    },
    FileTransfer {
        target: Cow<'a, str>,
    },
    Realise,
    CopyPaths,
    Builds,
    Build {
        target: Cow<'a, str>,
        source: Cow<'a, str>, // TODO: check
        val1: u64,
        val2: u64,
    },
    OptimiseStore,
    VerifyPaths,
    Substitute {
        source: Cow<'a, str>,
        target: Cow<'a, str>,
    },
    QueryPathInfo,
    PostBuildHook,
    BuildWaiting,
}

#[derive(Debug)]
pub enum ResultFields<'a> {
    BuildLogLine(Cow<'a, str>),
    SetPhase(&'a str),
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
pub enum Action<'a> {
    Msg {
        level: u8,
        msg: Cow<'a, str>,
    },
    Start {
        start_type: StartFields<'a>,
        id: BuildStepId,
        level: u8,
        parent: BuildStepId,
        text: Cow<'a, str>,
    },
    Result {
        id: BuildStepId,
        fields: ResultFields<'a>,
    },
    Stop {
        id: BuildStepId,
    },
}

impl<'a> Action<'a> {
    pub fn parse(s: &'a str) -> anyhow::Result<Self> {
        let raw: RawAction = serde_json::from_str(s).context("Could not parse raw JSON")?;
        raw.try_into().context("Could not convert raw action")
    }
}
