{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "chain-libs";
  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = "${name}/${subPkg}";};
in {
  chain-addr = mkSimplePkg "chain-addr";
  cardano-legacy-address = mkSimplePkg "cardano-legacy-address";
  chain-core = mkSimplePkg "chain-core";
  chain-crypto = mkSimplePkg "chain-crypto";
  chain-evm = mkSimplePkg "chain-evm";
  chain-impl-mockchain = mkSimplePkg "chain-impl-mockchain";
  chain-network = mkSimplePkg "chain-network";
  chain-ser = mkSimplePkg "chain-ser";
  chain-storage = mkSimplePkg "chain-storage";
  chain-time = mkSimplePkg "chain-time";
  chain-vote = mkSimplePkg "chain-vote";
  sparse-array = mkSimplePkg "sparse-array";
}
