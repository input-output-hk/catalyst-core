{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cardano-node.packages) cardano-cli;
  inherit (inputs.cells.jormungandr.packages) jcli;
  inherit (inputs.cells.catalyst-toolbox.packages) catalyst-toolbox;
  inherit (inputs.cells.voting-tools.packages) voting-tools;
  inherit (inputs.cells.voting-tools-rs.packages) voting-tools-rs;

  l = nixpkgs.lib // builtins;

  mkSimpleOperable = {
    name,
    runtimeInputs,
    args,
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
  snapshot-trigger-service = mkSimpleOperable {
    name = "snapshot-trigger-service";
    runtimeInputs = [
      jcli
      voting-tools
      voting-tools-rs
    ];
    args = {
      "--config" = "/secrets/snapshot-trigger-service.config";
    };
  };
}
