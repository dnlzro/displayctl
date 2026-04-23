{
  description = "List and toggle macOS displays from the command line";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forEachSystem (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "displayctl";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          meta = {
            description = "List and toggle macOS displays from the command line";
            homepage = "https://github.com/dnlzro/displayctl";
            license = pkgs.lib.licenses.mit;
            platforms = pkgs.lib.platforms.darwin;
            mainProgram = "displayctl";
          };
        };
      });

      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];
        };
      });
    };
}
