use std::iter;

#[derive(Debug)]
pub enum Program {
    Nix,
    NixCollectGarbage,
    NixOsRebuild,
    NixShell,
    Unknown(String),
}

#[derive(Debug)]
pub struct NixCommand {
    program: Program,
    args: Vec<String>,
}

impl NixCommand {
    pub fn program_str(&self) -> &str {
        match &self.program {
            Program::Nix => "nix",
            Program::NixCollectGarbage => "nix-collect-garbage",
            Program::NixOsRebuild => "nixos-rebuild",
            Program::NixShell => "nix-shell",
            Program::Unknown(cmd) => cmd,
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
            (Program::NixShell, _) | (Program::Nix, Some("repl" | "develop" | "shell"))
        )
    }

    fn extra_params(&self) -> impl Iterator<Item = &'static str> + '_ {
        let required: &[(&str, &[_])] = match (&self.program, self.args.as_slice()) {
            (Program::Nix | Program::NixOsRebuild, &[..]) => &[
                ("--print-build-logs", &[]),
                ("--log-format", &["internal-json"]),
            ],
            (Program::NixCollectGarbage | Program::NixShell, &[..]) => {
                &[("--log-format", &["internal-json"])]
            }
            (Program::Unknown(_), &[..]) => &[],
        };

        required
            .iter()
            .filter(move |(flag, _)| self.args.iter().all(|arg| arg != flag))
            .flat_map(move |(flag, vals)| iter::once(flag).chain(*vals))
            .copied()
    }
}

impl NixCommand {
    pub fn from_program_and_args(program: Program, args: impl Iterator<Item = String>) -> Self {
        Self {
            program,
            args: args.collect(),
        }
    }

    pub fn from_args(mut args: impl Iterator<Item = String>) -> Option<Self> {
        let program_str = args.next()?;

        let program = match program_str.as_str() {
            "nix" => Program::Nix,
            "nix-collect-garbage" => Program::NixCollectGarbage,
            "nixos-rebuild" => Program::NixOsRebuild,
            "nix-shell" => Program::NixShell,
            _ => Program::Unknown(program_str),
        };

        Some(Self::from_program_and_args(program, args))
    }
}
