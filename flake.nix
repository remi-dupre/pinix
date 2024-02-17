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
          manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          pinix =
            { rustPlatform
            , installShellFiles
            , lib
            }:
            rustPlatform.buildRustPackage {
              pname = manifest.package.name;
              version = manifest.package.version;
              src = ./.;
              nativeBuildInputs = [ installShellFiles ];

              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "console-0.16.0" = "sha256-dydgMgkqvBNW7eJntFIhBBWqOQnGnpQsVPRIBdqf0fY=";
                  "indicatif-0.17.8" = "sha256-KaTEGLU904ZVz2tEFZ/ls6B3LKCxhOVVuWSSLOifSAk=";
                };
              };

              meta = with lib; {
                description = "A Pacman inspired frontend for Nix";
                homepage = "https://github.com/remi-dupre/pinix";
                license = licenses.lgpl3;
                mainProgram = "pinix";
              };

              postInstall =
                let
                  wrappers = [ "nix" "nix-collect-garbage" "nixos-rebuild" "nix-shell" ];
                  install-wrappers =
                    lib.lists.forEach
                      wrappers
                      (nix-cmd:
                        let
                          pix-cmd = "pix" + (lib.strings.removePrefix "nix" nix-cmd);
                          pkg = pkgs.writeShellApplication {
                            name = pix-cmd;
                            text = ''pinix --pix-command ${nix-cmd} "$@"'';
                          };
                        in
                        ''
                          cat ${pkg}/bin/${pix-cmd} > $out/bin/${pix-cmd}
                          chmod +x $out/bin/${pix-cmd}
                        ''
                      );
                in
                lib.lists.foldl
                  (acc: line: acc + "\n" + line) ""
                  install-wrappers;

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
