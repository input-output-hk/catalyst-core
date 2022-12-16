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
      name = "${constants.registry}/jormungandr";
      tag = namespace;
      operable = cell.operables."jormungandr-${namespace}";
      debug = true;
    };
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOCI
