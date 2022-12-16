{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) constants;
  l = nixpkgs.lib // builtins;

  mkSimpleOperable = {
    name,
    runtimeInputs ? [],
    args ? [],
  }: let
    package = cell.packages.${name};
  in
    std.lib.ops.mkOperable {
      inherit package runtimeInputs;
      runtimeScript = std.lib.ops.mkOperableScript {
        inherit args package;
      };
    };
in {
  vit-servicing-station-server = mkSimpleOperable {
    name = "vit-servicing-station-server";
    args = {
      "--in-settings-file" = "/local/station-config.json";
    };
  };
}
