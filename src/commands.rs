use std::iter;

#[derive(Debug)]
pub enum BaseCommand {
    Nix,
    NixCollectGarbage,
    NixOsRebuild,
    NixShell,
    Unknown(String),
}

#[derive(Debug)]
pub struct NixCommand {
    program: BaseCommand,
    args: Vec<String>,
}

impl NixCommand {
    pub fn program_str(&self) -> &str {
        match &self.program {
            BaseCommand::Nix => "nix",
            BaseCommand::NixCollectGarbage => "nix-collect-garbage",
            BaseCommand::NixOsRebuild => "nixos-rebuild",
            BaseCommand::NixShell => "nix-shell",
            BaseCommand::Unknown(cmd) => cmd,
        }
    }

    pub fn params_unwrapped(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.args.iter().map(String::as_str)
    }

    pub fn params_wrapped(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.extra_params()
            .map(|s| s as _)
            .chain(self.args.iter().map(String::as_str))
    }

    pub fn is_repl(&self) -> bool {
        matches!(
            (&self.program, self.args.first().map(String::as_str)),
            (BaseCommand::NixShell, _) | (BaseCommand::Nix, Some("repl" | "develop" | "shell"))
        )
    }

    fn extra_params(&self) -> impl Iterator<Item = &'static str> + '_ {
        let required: &[(&str, &[_])] = match (&self.program, self.args.as_slice()) {
            (BaseCommand::Nix | BaseCommand::NixOsRebuild, &[..]) => &[
                ("--print-build-logs", &[]),
                ("--log-format", &["internal-json"]),
            ],
            (BaseCommand::NixCollectGarbage | BaseCommand::NixShell, &[..]) => {
                &[("--log-format", &["internal-json"])]
            }
            (BaseCommand::Unknown(_), &[..]) => &[],
        };

        required
            .iter()
            .filter(move |(flag, _)| self.args.iter().all(|arg| arg != flag))
            .flat_map(move |(flag, vals)| iter::once(flag).chain(*vals))
            .copied()
    }
}

impl NixCommand {
    pub fn parse_from_args() -> Option<Self> {
        let mut args = std::env::args().skip(1);
        let program = args.next()?;
        let args: Vec<_> = args.collect();

        let command = match program.as_str() {
            "nix" => BaseCommand::Nix,
            "nix-collect-garbage" => BaseCommand::NixCollectGarbage,
            "nixos-rebuild" => BaseCommand::NixOsRebuild,
            "nix-shell" => BaseCommand::NixShell,
            _ => BaseCommand::Unknown(program),
        };

        Some(Self {
            program: command,
            args,
        })
    }
}
