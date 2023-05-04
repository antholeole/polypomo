{
  description = "a polybar pomodoro widget";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
  let 
    cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
  in flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};
        
      in rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      }
    ) // {
        overlay = final: prev: {
            "${cargoToml.package.name}" = final.callPackage ./. { inherit naersk; };
      };
    };
}

