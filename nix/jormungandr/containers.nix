{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants;
  l = nixpkgs.lib // builtins;

  mkOCI = name: namespace:
    std.lib.ops.mkStandardOCI {
      name = "${constants.registry}/${name}-${namespace}";
      operable = cell.operables."${name}-${namespace}";
      debug = true;
    };
in
  {}
  // constants.mapToNamespaces {prefix = "jormungandr";} (mkOCI "jormungandr")
