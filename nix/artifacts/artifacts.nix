{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
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
  lib.mapToNamespaces "artifacts" mkArtifact
