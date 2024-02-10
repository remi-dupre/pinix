{
  description = "REST API for any Postgres database";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    with import nixpkgs { system = "x86_64-linux"; };

    pkgs.rustPlatform.buildRustPackage rec {
      pname = "pinix";
      version = "0.1.0";
      src = ./.;
      nativeBuildInputs = [ pkgs.installShellFiles ];


      cargoLock = {
        lockFile = ./Cargo.lock;
        outputHashes = {
          "console-0.16.0" = "sha256-t5hydPT3BXJqPl8zKieaId3KUltEbRPh2xmZMhy8Ut0=";
          "indicatif-0.17.8" = "sha256-8xrGqDb6iUDhdvY937XFqC3GIZpq4tPMZAkKm218c/0=";
        };
      };

      meta = with pkgs.lib; {
        description = "A Pacman inspired frontend for Nix";
        homepage = "https://github.com/remi-dupre/pinix";
        license = licenses.lgpl3;
        mainProgram = "pinix";
      };

      cargoSha256 = "sha256-6hKbAL3a1t1mHNUvvi65e/BkFoJQmvpZQlU58csmol4=";
    };
}
