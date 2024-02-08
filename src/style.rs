use std::fmt;
use std::time::Duration;

use console::style;
use indicatif::ProgressStyle;
use once_cell::sync::Lazy;
use regex::Regex;

pub const PROGRESS_WIDTH: u64 = 30;
pub const SPINNER_FREQ: Duration = Duration::from_millis(100);

pub static SPINNER_STR: Lazy<Vec<&str>> = Lazy::new(|| {
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

pub static PROGRESS_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    let p = "{spinner} {prefix} {wide_msg} {pos:>5}/{len:<6}";

    ProgressStyle::with_template(&format!("{p} [{{bar:{PROGRESS_WIDTH}}}] {{elapsed:>4}}"))
        .unwrap()
        .progress_chars("## ")
        .tick_strings(&SPINNER_STR)
});

pub static DOWNLOAD_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    let p = "{spinner} {prefix} {wide_msg} {binary_bytes_per_sec:^12} {bytes:^12}";

    ProgressStyle::with_template(&format!("{p} [{{bar:{PROGRESS_WIDTH}}}] {{elapsed:>4}}",))
        .unwrap()
        .progress_chars("## ")
        .tick_strings(&SPINNER_STR)
});

pub static LOGS_WINDOW_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(&style("{prefix} {wide_msg}").dim().to_string()).unwrap()
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
    .expect("invalid RegEx")
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

#[derive(Debug)]
pub struct MultiBar<'s, const N: usize>(pub [(&'s str, u64); N]);

impl<const N: usize> fmt::Display for MultiBar<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &(c, len) in &self.0 {
            for _ in 0..len {
                f.write_str(c)?;
            }
        }

        Ok(())
    }
}
