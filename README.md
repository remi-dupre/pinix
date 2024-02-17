# Progress In Nix

Pinix is a Pacman inspired frontend for Nix. It wraps a regular Nix command and
replaces the output with a more modern and informative interface.

[![asciicast](demo-gif)](demo-ascii)

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

### Using Cargo

You can also install pinix from sources by using [cargo](cargo):

```shell
cargo install pinix

# This will only install the main binary so you might want to add aliases for
# common nix commands.
alias pix="pinix --command nix"
alias pix-shell="pinix --command nix-shell"
alias pixos-rebuild="pinix --command nixos-rebuild"
```

## Usage

## Similar Tools

## How it works


[cargo]: https://doc.rust-lang.org/cargo/
[demo-ascii]: https://asciinema.org/a/641197
[demo-gif]: https://github.com/remi-dupre/pinix/assets/1173464/6ab7ceb4-2ab3-41b8-84d0-78c6278d6d55
