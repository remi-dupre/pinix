use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "df_false")]
    pub debug: bool,

    #[serde(default)]
    pub summary: ConfigSummary,

    #[serde(default)]
    pub log_history: ConfigLogHistory,

    #[serde(default)]
    pub log_window: ConfigLogWindow,
}

// Summary

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ConfigSummary {
    #[serde(default = "df_false")]
    pub download: bool,
}

impl Default for ConfigSummary {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

// Log History

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ConfigLogHistory {
    #[serde(default = "df_log_history_size")]
    pub size: usize,

    #[serde(default = "df_log_history_size")]
    pub failure_size: usize,
}

impl Default for ConfigLogHistory {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

fn df_log_history_size() -> usize {
    1000
}

// Log Window

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ConfigLogWindow {
    #[serde(default = "df_log_window_size")]
    pub size: usize,
}

impl Default for ConfigLogWindow {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

fn df_log_window_size() -> usize {
    10
}

// Common Defaults

fn df_false() -> bool {
    false
}

// /// TOML doesn't support null values, so options are implemented through "false"
// fn deserialize_opt_bool<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<Option<T>, D::Error>
// where
//     D: de::Deserializer<'de>,
// {
//     #[derive(Deserialize)]
//     #[serde(untagged)]
//     enum Either<T> {
//         Value(T),
//         Boolean(bool),
//     }
//
//     match Either::<T>::deserialize(deserializer)? {
//         Either::Value(val) => Ok(Some(val)),
//         Either::Boolean(false) => Ok(None),
//         Either::Boolean(true) => Err(de::Error::invalid_value(
//             de::Unexpected::Bool(true),
//             &"a value or false",
//         )),
//     }
// }
