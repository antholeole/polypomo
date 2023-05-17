{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      rustPkg = flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };

          naersk' = pkgs.callPackage naersk { };

        in {
          defaultPackage = naersk'.buildPackage {
            src = ./.;
          };

          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ rustc cargo ];
          };
        }
      );
    in
    {
      defaultPackage = rustPkg.defaultPackage;
      devShell = rustPkg.devShell;
      overlays.default = prev: final: {
        polydoro = rustPkg.defaultPackage."${final.system}";
      };
    };
}
