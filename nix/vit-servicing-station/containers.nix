{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants lib;
  l = nixpkgs.lib // builtins;

  mkOCI = namespace: let
    # TODO: fix git rev
    # rev =
    #   if (inputs.self.rev != "not-a-commit")
    #   then inputs.self.rev
    #   else "dirty";
  in
    std.lib.ops.mkStandardOCI {
      name = "${constants.registry}/vit-servicing-station-server";
      tag = namespace;
      operable = cell.operables."vit-servicing-station-server-${namespace}";
      debug = true;
    };
in
  {}
  // lib.mapToNamespaces "vit-servicing-station-server" mkOCI
