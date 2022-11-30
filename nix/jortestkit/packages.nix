{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.automation.lib) mkPackage;
  l = nixpkgs.lib // builtins;

  name = "jortestkit";
  root = inputs.self + "/src/${name}";

  mkSimplePkg = subPkg: mkPackage {pkgPath = root + "/${subPkg}";};
in {
  jortestkit = mkSimplePkg "";
}
