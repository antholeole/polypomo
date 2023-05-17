{
  description = "Foo Bar";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;
    in
    {
      overlays.default = {
        polydoro = nixpkgs.rustPlatform.buildRustPackage rec {
          pname = manifest.name;
          version = manifest.version;
          cargoLock.lockFile = ./Cargo.lock;
          src = nixpkgs.lib.cleanSource ./.;
        };
      };
    };
}
