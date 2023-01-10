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
  jormungandr = lib.mkPackage {
    pkgPath = "${name}/jormungandr";
    cargoOptions = [
      "--features"
      "prometheus-metrics"
    ];
  };
  jcli = mkSimplePkg "jcli";
}
