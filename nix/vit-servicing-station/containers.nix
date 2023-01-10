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
        name = "${constants.registry}/vit-servicing-station-server";
        operable = cell.operables."vit-servicing-station-server-${namespace}";
        debug = true;
      }
      # Default to using output hash as the tag if the repo is dirty
      // l.optionalAttrs (rev != "") {tag = "${rev}-${namespace}";});
in
  {}
  // lib.mapToNamespaces "vit-servicing-station-server" mkOCI
