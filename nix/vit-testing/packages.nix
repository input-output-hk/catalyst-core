{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "vit-testing";
  root = inputs.self + "/src/${name}";

  mkVitPkg = subPkg:
    lib.mkPackage {
      pkgPath = root + "/${subPkg}";
      nativeBuildInputs = with nixpkgs; [
        postgresql.lib
      ];
    };
in {
  mainnet-tools = mkVitPkg "mainnet-tools";
  snapshot-trigger-service = mkVitPkg "snapshot-trigger-service";
  vitup = mkVitPkg "vitup";
}
