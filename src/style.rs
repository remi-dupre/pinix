use std::time::Duration;

use console::style;
use indicatif::ProgressStyle;
use once_cell::sync::Lazy;
use regex::Regex;

pub const SPINNER_FREQ: Duration = Duration::from_millis(100);

static SPINNER_STR: Lazy<Vec<&str>> = Lazy::new(|| {
    static AS_STRINGS: Lazy<Vec<String>> = Lazy::new(|| {
        let mut tick_strings: Vec<_> = "⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈"
            .chars()
            .map(|c| style(c).blue().to_string())
            .collect();

        tick_strings.push(style("✓").green().to_string());
        tick_strings
    });

    AS_STRINGS.iter().map(String::as_str).collect()
});

static _SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{spinner} {prefix} {wide_msg} {elapsed:>4}")
        .unwrap()
        .tick_strings(&SPINNER_STR)
});

pub static PROGRESS_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(
        "{spinner} {prefix} {wide_msg} {pos}/{len:<8} [{bar:40}] {elapsed:>4}",
    )
    .unwrap()
    .progress_chars("## ")
    .tick_strings(&SPINNER_STR)
});

pub static DOWNLOAD_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(
        "{spinner} {prefix} {wide_msg} {binary_bytes_per_sec:<14} {bytes:<12} [{bar:40}] {elapsed:>4}",
    )
    .unwrap()
    .progress_chars("## ")
    .tick_strings(&SPINNER_STR)
});

static MATCH_BUILD_TARGET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"^",
        r"(?P<prefix>\/nix\/store\/[a-z0-9]+)-",
        r"(?P<name>.*?)",
        r"(?:-(?P<version>\d{4}-\d{2}-\d{2}|[ab\d.]+))?",
        r"(?:\.drv)?",
        r"$",
    ))
    .unwrap()
});

fn match_build_target(raw_str: &str) -> Option<(&str, &str, Option<&str>)> {
    let matched = MATCH_BUILD_TARGET.captures(raw_str)?;

    Some((
        matched
            .name("prefix")
            .expect("no name found in build target")
            .as_str(),
        matched
            .name("name")
            .expect("no name found in build target")
            .as_str(),
        matched.name("version").map(|m| m.as_str()),
    ))
}

pub fn format_short_build_target(raw_str: &str) -> String {
    let Some((_prefix, name, version)) = match_build_target(raw_str) else {
        return style(raw_str).yellow().to_string();
    };

    let mut result = style(name).blue().to_string();

    if let Some(version) = version {
        result = format!("{result}-{}", style(version));
    }

    result
}

pub fn format_build_target(raw_str: &str) -> String {
    let Some((prefix, name, version)) = match_build_target(raw_str) else {
        return style(raw_str).yellow().to_string();
    };

    let mut result = format!("{prefix}-{}", style(name).blue());

    if let Some(version) = version {
        result = format!("{result}-{version}");
    }

    result
}
