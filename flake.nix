{
  description = "a polybar pomodoro widget";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, utils }: 
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages = {
          default = pkgs.stdenv.mkDerivation {
            name = "polypomo";
            propagatedBuildInputs = [
              pkgs.python3_11
            ];
            dontUnpack = true;
            installPhase = "install -Dm755 ${./polypomo.py} $out/bin/polypomo";
          };
        }; 
      }
  );
}