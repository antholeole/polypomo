{
  description = "A Pomodoro widget for Polybar.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      outputsBySystem = flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };

          naersk' = pkgs.callPackage naersk { };
        in
        naersk'.buildPackage {
          src = ./.;
        });
    in
    {
      overlays.default = final: prev: {
        polydoro = outputsBySystem."${prev.system}";
      };
    };
}
