{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "displayctl";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "List and toggle macOS displays from the command line";
    homepage = "https://github.com/dnlzro/displayctl";
    license = pkgs.lib.licenses.mit;
    platforms = pkgs.lib.platforms.darwin;
    mainProgram = "displayctl";
  };
}
