{
  description = "a polybar pomodoro widget";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: {
    overlay = (self: super: {
        polypomo = nixpkgs.stdenv.mkDerivation {
            name = "polypomo";
            propagatedBuildInputs = [
              nixpkgs.python3_11
            ];
            dontUnpack = true;
            installPhase = "install -Dm755 ${./polypomo.py} $out/bin/polypomo";
        };
    });
  };
}