{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "vit-testing";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  mainnet-tools = mkSimplePkg "mainnet-tools";
  snapshot-trigger-service = mkSimplePkg "snapshot-trigger-service";
  vitup = mkSimplePkg "vitup";
}
