{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.automation.lib) mkPackage;
  l = nixpkgs.lib // builtins;

  name = "catalyst-toolbox";
  root = inputs.self + "/src/${name}";

  mkSimplePkg = subPkg: mkPackage {pkgPath = root + "/${subPkg}";};
in {
  catalyst-toolbox = mkPackage {
    pkgPath = root + "/catalyst-toolbox";
    nativeBuildInputs = with nixpkgs; [
      postgresql.lib
    ];
  };
  snapshot-lib = mkSimplePkg "snapshot-lib";
}
