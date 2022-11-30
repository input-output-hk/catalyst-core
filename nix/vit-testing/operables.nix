{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cardano-node.packages) cardano-cli;
  inherit (inputs.cells.jormungandr.packages) jcli;
  inherit (inputs.cells.catalyst-toolbox.packages) catalyst-toolbox;
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
  registration-service = mkSimpleOperable {
    name = "registration-service";
    runtimeInputs = [
      catalyst-toolbox
      jcli
      cardano-cli
      voting-tools-rs
    ];
    args = {
      "--config" = "/secrets/registration-service.config";
    };
  };
  registration-verify-service = mkSimpleOperable {
    name = "registration-verify-service";
    runtimeInputs = [
      jcli
    ];
    args = {
      "--config" = "/secrets/registration-verify-service.config";
    };
  };
  snapshot-trigger-service = mkSimpleOperable {
    name = "snapshot-trigger-service";
    runtimeInputs = [
      jcli
    ];
    args = {
      "--config" = "/secrets/snapshot-trigger-service.config";
    };
  };
}
