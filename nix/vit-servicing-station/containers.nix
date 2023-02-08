{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants lib;
  l = nixpkgs.lib // builtins;

  operable = cell.operables.vit-servicing-station-server;
in {
  vit-servicing-station-server = let
    rev =
      if (inputs.self.rev != "not-a-commit")
      then inputs.self.rev
      else "";
  in
    std.lib.ops.mkStandardOCI ({
        inherit operable;
        name = "${constants.registry}/vit-servicing-station-server";
        debug = true;
      }
      # Include common container setup
      // lib.containerCommon
      # Default to using output hash as the tag if the repo is dirty
      // l.optionalAttrs (rev != "") {tag = rev;});
}
