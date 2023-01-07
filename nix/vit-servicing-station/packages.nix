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
  vit-servicing-station-server = lib.mkPackage {
    pkgPath = "${name}/vit-servicing-station-server";
    postInstall = ''
      cp -r src/vit-servicing-station/vit-servicing-station-lib/migrations/postgres $out/migrations
    '';
  };
}
