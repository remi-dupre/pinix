#[derive(Clone, Debug)]
pub enum WrappedProgram {
    Nix,
    NixCollectGarbage,
    NixOsRebuild,
    NixShell,
    Unknown(String),
}

impl WrappedProgram {
    pub fn as_str(&self) -> &str {
        match self {
            WrappedProgram::Nix => "nix",
            WrappedProgram::NixCollectGarbage => "nix-collect-garbage",
            WrappedProgram::NixOsRebuild => "nixos-rebuild",
            WrappedProgram::NixShell => "nix-shell",
            WrappedProgram::Unknown(path) => path.as_str(),
        }
    }
}

impl From<String> for WrappedProgram {
    fn from(value: String) -> Self {
        match value.as_str() {
            "nix" => Self::Nix,
            "nix-collect-garbage" => Self::NixCollectGarbage,
            "nixos-rebuild" => Self::NixOsRebuild,
            "nix-shell" => Self::NixShell,
            _ => Self::Unknown(value),
        }
    }
}

impl std::fmt::Display for WrappedProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
