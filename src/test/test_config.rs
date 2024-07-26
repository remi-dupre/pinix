use std::path::PathBuf;

use crate::wrapper::config::{Config, ConfigLogHistory, ConfigLogWindow, ConfigSummary};

#[test]
fn default_parsing() -> anyhow::Result<()> {
    let _: Config = toml::from_str("")?;
    Ok(())
}

#[test]
fn simple_parsing() -> anyhow::Result<()> {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("src/test/data/config-1.toml");
    let raw = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&raw)?;

    assert_eq!(
        config,
        Config {
            debug: false,
            summary: ConfigSummary { download: true },
            log_history: ConfigLogHistory {
                size: 5,
                failure_size: 30
            },
            log_window: ConfigLogWindow { size: 10 }
        }
    );

    Ok(())
}

#[test]
fn unknown_field() -> anyhow::Result<()> {
    let val1 = toml::toml! { unknown = true };

    let val2 = toml::toml! {
        [log-history]
        unknown = true
    };

    let val3 = toml::toml! {
        [unknown]
        unknown = true
    };

    assert!(val1.try_into::<Config>().is_err());
    assert!(val2.try_into::<Config>().is_err());
    assert!(val3.try_into::<Config>().is_err());
    Ok(())
}
