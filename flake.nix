{
  description = "A Pacman inspired frontend for Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      lib = nixpkgs.lib;
      allSystems = lib.systems.flakeExposed;
      forAllSystems = lib.genAttrs allSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pinix =
            { rustPlatform
            , installShellFiles
            , lib
            }:
            rustPlatform.buildRustPackage {
              pname = "pinix";
              version = "0.1.0";
              src = ./.;
              nativeBuildInputs = [ installShellFiles ];

              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "console-0.16.0" = "sha256-t5hydPT3BXJqPl8zKieaId3KUltEbRPh2xmZMhy8Ut0=";
                  "indicatif-0.17.8" = "sha256-8xrGqDb6iUDhdvY937XFqC3GIZpq4tPMZAkKm218c/0=";
                };
              };

              meta = with lib; {
                description = "A Pacman inspired frontend for Nix";
                homepage = "https://github.com/remi-dupre/pinix";
                license = licenses.lgpl3;
                mainProgram = "pinix";
              };

              cargoSha256 = "sha256-6hKbAL3a1t1mHNUvvi65e/BkFoJQmvpZQlU58csmol4=";
            };
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          pinix = pkgs.callPackage pinix { };
          default = self.packages.${system}.pinix;
        }
      );
    };
}
