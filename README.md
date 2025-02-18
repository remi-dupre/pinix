# Progress In Nix

![Crates.io License](https://img.shields.io/crates/l/pinix)
![Crates.io Version](https://img.shields.io/crates/v/pinix)

Pinix is a Pacman inspired frontend for Nix. It wraps a regular Nix command and
replaces the output with a more modern and informative interface.

[![asciicast][demo-gif]][demo-ascii]

It _should_ work transparently for most commands, including when an interactive
shell is spawned.

## Installation

### Using Nix

The repository defines a flake, so you can get the _pinix_ package available by
adding it to your _flake.nix_:

```nix
inputs = {
  pinix.url = github:remi-dupre/pinix;
};
```

```nix
{ config, pkgs, lib, inputs, ... }:
 
{
  environment.systemPackages = [
    inputs.pinix.packages.${system}.default
  ];
}
```

### Using Cargo

You can also install pinix from sources by using [cargo][cargo]:

```shell
cargo install pinix

# This will only install the main binary so you might want to add aliases for
# common nix commands.
alias pix="pinix --command nix"
alias pix-shell="pinix --command nix-shell"
alias pixos-rebuild="pinix --command nixos-rebuild"
```

## Usage

The nix package provides you with drop-in replacements for common nix commands:

```shell
$ pix-shell -p htop
$ pixos-rebuild switch --flake .
```

Pinix has its how set of parameters, all prefixed with `--pix-`, they must be
specified **before** any regular parameter. You can get list supported
parameters through the help message:

```
$ pinix --pix-help
Wrap a Nix command to display rich logs while it is running

Usage: pinix [OPTIONS] [EXT]...

Arguments:
  [EXT]...  Arguments forwared to actual Nix command

Options:
      --pix-help               Display this help message
      --pix-command <COMMAND>  Specify the nix command that must be run
      --pix-debug              Display a debug bar
      --pix-log-downloads      Display a log line when a download is finished
      --pix-record <RECORD>    Save timestamped logs to a file
```

If you want to run a command for which you don't have an alias available you can
call `pinix` followed by your regular command:

```shell
$ pinix nix-shell -p htop
```

## Similar Tools

I'm not the first one who tried to improve nix output. Here are the tools that I
know of:

- **[nix-output-monitor][tool-nom]**: Pipe your nix-build output through the
  nix-output-monitor a.k.a nom to get additional information while building.
- **[nvd][tool-nvd]**: Nix/NixOS package version diff tool.
- **[#4296][tool-native]**: Some old suggestion for a more riche native progress
  indicator in nix.



[cargo]: https://doc.rust-lang.org/cargo/
[demo-ascii]: https://asciinema.org/a/641197
[demo-gif]: https://github.com/remi-dupre/pinix/assets/1173464/6ab7ceb4-2ab3-41b8-84d0-78c6278d6d55
[tool-nom]: https://github.com/maralorn/nix-output-monitor
[tool-nvd]: https://gitlab.com/khumba/nvd
[tool-native]: https://github.com/NixOS/nix/pull/4296
