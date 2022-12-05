{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  name = "voting-center-backend";
  operable = cell.operables.voting-center-backend;

  mkOCI = name: let
    operable = cell.operables.${name};
  in
    std.lib.ops.mkStandardOCI {
      inherit name operable;
    };
in {
  snapshot-trigger-service = mkOCI "snapshot-trigger-service";
}
