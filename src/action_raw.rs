use std::borrow::Cow;

use anyhow::Context;
use serde::Deserialize;
use serde_json::value::RawValue;
use serde_repr::Deserialize_repr;

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RawActionType {
    Msg,
    Start,
    Result,
    Stop,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum RawEnumType {
    FileLinked = 100,
    BuildLogLine = 101,
    UntrustedPath = 102,
    CorruptedPath = 103,
    SetPhase = 104,
    Progress = 105,
    SetExpected = 106,
    PostBuildLogLine = 107,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub struct RawAction<'a> {
    #[serde(rename = "action")]
    pub action_type: RawActionType,
    pub level: Option<u8>,
    pub msg: Option<Cow<'a, str>>,
    pub id: Option<BuildStepId>,
    pub parent: Option<BuildStepId>,
    pub text: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub fields: Option<&'a RawValue>,
    #[serde(rename = "type")]
    pub any_type: Option<u8>,
}

impl<'a> TryFrom<RawAction<'a>> for Action<'a> {
    type Error = anyhow::Error;

    fn try_from(val: RawAction<'a>) -> Result<Self, Self::Error> {
        let missing = |field: &'static str| anyhow::anyhow!("missing field `{field}`");

        let action = match val.action_type {
            RawActionType::Msg => Action::Msg {
                level: val.level.ok_or_else(|| missing("level"))?,
                msg: val.msg.ok_or_else(|| missing("msg"))?,
            },

            RawActionType::Start => {
                let start_type: ActionType =
                    serde_json::from_value(val.any_type.ok_or_else(|| missing("type"))?.into())
                        .context("invalid type")?;

                let start_type = match start_type {
                    ActionType::Unknown => StartFields::Unknown,
                    ActionType::CopyPath => {
                        let raw_fields = val.fields.ok_or_else(|| missing("fields"))?.get();

                        let (path, origin, destination) =
                            serde_json::from_str(raw_fields).context("invalid fields")?;

                        StartFields::CopyPath {
                            path,
                            origin,
                            destination,
                        }
                    }
                    ActionType::FileTransfer => {
                        let raw_fields = val.fields.ok_or_else(|| missing("fields"))?.get();

                        let [target] =
                            serde_json::from_str(raw_fields).context("invalid fields")?;

                        StartFields::FileTransfer { target }
                    }
                    ActionType::Realise => StartFields::Realise,
                    ActionType::CopyPaths => StartFields::CopyPaths,
                    ActionType::Builds => StartFields::Builds,
                    ActionType::Build => {
                        let raw_fields = val.fields.ok_or_else(|| missing("fields"))?.get();

                        let (target, source, val1, val2) =
                            serde_json::from_str(raw_fields).context("invalid fields")?;

                        StartFields::Build {
                            target,
                            source,
                            val1,
                            val2,
                        }
                    }
                    ActionType::OptimiseStore => StartFields::OptimiseStore,
                    ActionType::VerifyPaths => StartFields::VerifyPaths,
                    ActionType::Substitute => {
                        let raw_fields = val.fields.ok_or_else(|| missing("fields"))?.get();

                        let (source, target) =
                            serde_json::from_str(raw_fields).context("invalid fields")?;

                        StartFields::Substitute { source, target }
                    }
                    ActionType::QueryPathInfo => StartFields::QueryPathInfo,
                    ActionType::PostBuildHook => StartFields::PostBuildHook,
                    ActionType::BuildWaiting => StartFields::BuildWaiting,
                };

                Action::Start {
                    start_type,
                    id: val.id.ok_or_else(|| missing("id"))?,
                    level: val.level.ok_or_else(|| missing("level"))?,
                    parent: val.parent.ok_or_else(|| missing("parent"))?,
                    text: val.text.ok_or_else(|| missing("text"))?,
                }
            }

            RawActionType::Result => {
                let raw_fields = val.fields.ok_or_else(|| missing("fields"))?.get();

                let fields = match val.any_type.ok_or_else(|| missing("type"))? {
                    100 => todo!("FileLinked({raw_fields})"),
                    101 => {
                        let [line] = serde_json::from_str(raw_fields).context("invalid fields")?;
                        ResultFields::BuildLogLine(line)
                    }
                    102 => todo!("UntrustedPath({raw_fields})"),
                    103 => todo!("CorruptedPath({raw_fields})"),
                    104 => {
                        let [phase] = serde_json::from_str(raw_fields).context("invalid fields")?;
                        ResultFields::SetPhase(phase)
                    }
                    105 => {
                        let (done, expected, running, failed) =
                            serde_json::from_str(raw_fields).context("invalid fields")?;

                        ResultFields::Progress {
                            done,
                            expected,
                            running,
                            failed,
                        }
                    }
                    106 => {
                        let (action, expected) = serde_json::from_str(raw_fields).unwrap();
                        ResultFields::SetExpected { action, expected }
                    }
                    107 => todo!("PostBuildLogLine({raw_fields})"),
                    v => anyhow::bail!("Unknown result type `{v}`"),
                };

                Action::Result {
                    id: val.id.ok_or_else(|| missing("id"))?,
                    fields,
                }
            }

            RawActionType::Stop => Action::Stop {
                id: val.id.ok_or_else(|| missing("id"))?,
            },
        };

        Ok(action)
    }
}
