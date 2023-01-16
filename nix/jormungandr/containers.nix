{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants lib;
  l = nixpkgs.lib // builtins;

  mkOCI = namespace: let
    rev =
      if (inputs.self.rev != "not-a-commit")
      then inputs.self.rev
      else "";
  in
    std.lib.ops.mkStandardOCI ({
        name = "${constants.registry}/jormungandr";
        operable = cell.operables."jormungandr-${namespace}";
        debug = true;
      }
      # Include common container setup
      // lib.containerCommon
      # Default to using output hash as the tag if the repo is dirty
      // l.optionalAttrs (rev != "") {tag = "${rev}-${namespace}";});
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOCI
