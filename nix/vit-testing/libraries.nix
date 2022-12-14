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
  iapyx = mkSimplePkg "iapyx";
  integration-tests = mkSimplePkg "integration-tests";
  mainnet-lib = mkSimplePkg "mainnet-lib";
  scheduler-service-lib = mkSimplePkg "scheduler-service-lib";
  signals-handler = mkSimplePkg "signals-handler";
  valgrind = mkSimplePkg "valgrind";
}
