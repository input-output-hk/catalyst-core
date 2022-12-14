{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants;
  l = nixpkgs.lib // builtins;

  mkOCI = name: let
    operable = cell.operables.${name};
  in
    std.lib.ops.mkStandardOCI {
      inherit operable;
      name = "${constants.registry}/${name}";
    };
in {
  snapshot-trigger-service = mkOCI "snapshot-trigger-service";
}
