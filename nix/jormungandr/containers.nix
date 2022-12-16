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
      name = "${constants.registry}/jormungandr";
      tag = "${rev}-${namespace}";
      operable = cell.operables."jormungandr-${namespace}";
      debug = true;
    };
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOCI
