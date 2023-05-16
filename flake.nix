{
  description = "a polybar pomodoro widget";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      pkgName = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.name;
      naersk' = nixpkgs.callPackage naersk { };
    in
    flake-utils.lib.simpleFlake
      {
        inherit self nixpkgs;
        name = pkgName;

        shell = { pkgs, ... }: pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      } // {
      overlay.default = final: prev: {
        "${pkgName}" = naersk'.buildPackage {
          src = ./.;
        };
      };
    };
}

