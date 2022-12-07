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
  iapyx = mkVitPkg "iapyx";
  integration-tests = mkVitPkg "integration-tests";
  mainnet-lib = mkVitPkg "mainnet-lib";
  scheduler-service-lib = mkVitPkg "scheduler-service-lib";
  signals-handler = mkVitPkg "signals-handler";
  valgrind = mkVitPkg "valgrind";
}
