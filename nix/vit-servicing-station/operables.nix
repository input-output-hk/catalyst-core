{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  package = cell.packages.vit-servicing-station-server;

  mkVitOperable = namespace: let
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
  // lib.mapToNamespaces "vit-servicing-station-server" mkVitOperable
