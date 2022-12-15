{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "voting-tools-rs";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  voting-tools-rs = mkSimplePkg "";
}
