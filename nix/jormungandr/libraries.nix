{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "jormungandr";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  blockchain = mkSimplePkg "modules/blockchain";
  explorer = mkSimplePkg "explorer";
  hersir = mkSimplePkg "testing/hersir";
  jormungandr-lib = mkSimplePkg "jormungandr-lib";
  loki = mkSimplePkg "testing/loki";
  mjolnir = mkSimplePkg "testing/mjolnir";
  settings = mkSimplePkg "modules/settings";
  thor = mkSimplePkg "testing/thor";
}
