{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants;
  l = nixpkgs.lib // builtins;

  mkArtifact = namespace:
    nixpkgs.stdenv.mkDerivation {
      name = "artifacts-${namespace}";
      src = ./files;
      installPhase = ''
        mkdir $out
        cp -R ./${namespace}/* $out
      '';
    };
in
  constants.mapToNamespaces {prefix = "artifacts";} mkArtifact
