{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "vit-servicing-station";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  vit-servicing-station-lib = mkSimplePkg "vit-servicing-station-lib";
  vit-servicing-station-tests = mkSimplePkg "vit-servicing-station-tests";
}
