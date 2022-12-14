{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) constants;
  l = nixpkgs.lib // builtins;

  mkVitOperable = package: namespace: let
    artifacts' = artifacts."artifacts-${namespace}";
  in
    std.lib.ops.mkOperable {
      inherit package;
      runtimeInputs = [
        artifacts'
      ];
      runtimeScript = std.lib.ops.mkOperableScript {
        inherit package;
        args = {
          "--in-settings-file" = "/local/station-config.json";
        };
      };
    };
in
  {}
  // constants.mapToNamespaces {prefix = "vit-servicing-station-server";} (mkVitOperable cell.packages.vit-servicing-station-server)
