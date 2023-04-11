{
  description = "a polybar pomodoro widget";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    overlay = final: prev: {
        polypomo = final.mkDerivation {
            name = "polypomo";
            propagatedBuildInputs = [
              final.python3_11
            ];
            dontUnpack = true;
            installPhase = "install -Dm755 ${./polypomo.py} $out/bin/polypomo";
        };
  };
  };
}