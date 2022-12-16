{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants lib;
  l = nixpkgs.lib // builtins;

  mkOCI = namespace: let
    rev =
      if (inputs.self ? rev)
      then inputs.self.rev
      else "dirty";
  in
    std.lib.ops.mkStandardOCI {
      name = "${constants.registry}/vit-servicing-station-server";
      tag = "${rev}-${namespace}";
      operable = cell.operables."vit-servicing-station-server-${namespace}";
      debug = true;
    };
in
  {}
  // lib.mapToNamespaces "vit-servicing-station-server" mkOCI
