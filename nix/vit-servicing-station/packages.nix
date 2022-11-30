{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.automation.lib) mkPackage;
  l = nixpkgs.lib // builtins;

  name = "vit-servicing-station";
  root = inputs.self + "/src/${name}";

  mkVitPkg = subPkg:
    mkPackage {
      pkgPath = root + "/${subPkg}";
      nativeBuildInputs = with nixpkgs; [
        postgresql.lib
      ];
    };
in {
  vit-servicing-station-cli = mkVitPkg "vit-servicing-station-cli";
  vit-servicing-station-lib = mkVitPkg "vit-servicing-station-lib";
  vit-servicing-station-server = mkVitPkg "vit-servicing-station-server";
  vit-servicing-station-tests = mkVitPkg "vit-servicing-station-tests";
}
