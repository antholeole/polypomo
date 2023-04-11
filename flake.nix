{
  description = "a polybar pomodoro widget";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    overlay = final: prev: {
        polypomo = final.stdenv.mkDerivation {
            name = "polypomo";
            propagatedBuildInputs = [
              final.python311
            ];
            dontUnpack = true;
            installPhase = "install -Dm755 ${./polypomo} $out/bin/polypomo";
        };
    };
  };
}