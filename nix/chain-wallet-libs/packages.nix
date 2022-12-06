{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "chain-wallet-libs";
  root = inputs.self + "/src/${name}";

  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = root + "/${subPkg}";};
in {
  bip39 = mkSimplePkg "bip39";
  chain-path-derivation = mkSimplePkg "chain-path-derivation";
  hdkeygen = mkSimplePkg "hdkeygen";
  symmetric-cipher = mkSimplePkg "symmetric-cipher";
  wallet = mkSimplePkg "wallet";
  wallet-c = mkSimplePkg "bindings/wallet-c";
  wallet-core = mkSimplePkg "bindings/wallet-core";
  wallet-wasm-js = mkSimplePkg "bindings/wallet-wasm-js";
}
