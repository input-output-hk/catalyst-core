{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "catalyst-toolbox";
  root = inputs.self + "/src/${name}";

  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = root + "/${subPkg}";};
in {
  snapshot-lib = mkSimplePkg "snapshot-lib";
}
