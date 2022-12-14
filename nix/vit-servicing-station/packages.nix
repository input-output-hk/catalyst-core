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
  vit-servicing-station-cli = mkSimplePkg "vit-servicing-station-cli";
  vit-servicing-station-server = mkSimplePkg "vit-servicing-station-server";
}
