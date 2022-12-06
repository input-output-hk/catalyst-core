{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.voting-tools.packages) voting-tools;

  l = nixpkgs.lib // builtins;
in {
  inherit voting-tools;
}
