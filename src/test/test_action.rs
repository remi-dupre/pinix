use crate::action::{Action, ActionType, ResultFields, StartFields};

#[test]
fn test_parse_start() -> anyhow::Result<()> {
    let act = Action::parse(concat!(
        r#"{"action":"start","id":3239822680391680,"level":6,"parent":0,"text":"querying info ab"#,
        r#"out missing paths","type":0}"#,
    ))?;

    assert_eq!(
        act,
        Action::Start {
            start_type: StartFields::Unknown,
            id: 3239822680391680.into(),
            level: 6,
            parent: 0.into(),
            text: "querying info about missing paths".into(),
        }
    );

    Ok(())
}

#[test]
fn test_parse_result() -> anyhow::Result<()> {
    let act = Action::parse(
        r#"{"action":"result","fields":[101,4242],"id":3239822680391681,"type":106}"#,
    )?;

    assert_eq!(
        act,
        Action::Result {
            id: 3239822680391681.into(),
            fields: ResultFields::SetExpected {
                action: ActionType::FileTransfer,
                expected: 4242,
            }
        }
    );

    Ok(())
}

#[test]
fn test_parse_stop() -> anyhow::Result<()> {
    let act = Action::parse(r#"{"action":"stop","id":3239822680391838}"#)?;

    assert_eq!(
        act,
        Action::Stop {
            id: 3239822680391838.into()
        }
    );

    Ok(())
}
