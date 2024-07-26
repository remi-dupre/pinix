use std::fmt::{self, Display};
use std::sync::LazyLock;

use console::style;
use indicatif::ProgressStyle;
use regex::Regex;

static MATCH_BUILD_TARGET: LazyLock<Regex> = LazyLock::new(|| {
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

impl<'s, const N: usize> MultiBar<'s, N> {
    /// Total length of the bar
    pub(crate) fn length(&self) -> u64 {
        self.0.iter().map(|(_, len)| *len).sum()
    }

    /// Transforms the bar to be of target size
    pub(crate) fn scale(&self, size: u64) -> Self {
        let length = std::cmp::max(self.length(), 1);
        let mut prev_prop = 0;
        let mut curr_prop = 0;

        let inner = self.0.map(|(c, len)| {
            curr_prop += len;
            let nb_chars = size * curr_prop / length - size * prev_prop / length;
            prev_prop = curr_prop;
            (c, nb_chars)
        });

        Self(inner)
    }
}

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

pub fn template_style<R1, R2>(
    size: u16,
    show_duration: bool,
    main: impl FnOnce(u16) -> R1,
    bar: impl FnOnce(u16) -> R2,
) -> ProgressStyle
where
    R1: Display,
    R2: Display,
{
    let bar_size = size / 3;

    let (main_size, elapsed) = {
        if size > 90 {
            let duration_template = {
                if show_duration {
                    style(" {elapsed:<5}").dim()
                } else {
                    style("      ")
                }
            };

            ((size - bar_size - 6), duration_template)
        } else {
            ((size - bar_size), style(""))
        }
    };

    ProgressStyle::with_template(&format!("{}{}{elapsed}", main(main_size), bar(bar_size)))
        .expect("invalid template")
        .progress_chars("## ")
}
