{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "catalyst-toolbox";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  catalyst-toolbox = lib.mkPackage {
    pkgPath = "${name}/catalyst-toolbox";
    nativeBuildInputs = with nixpkgs; [
      postgresql.lib
    ];
  };
}
