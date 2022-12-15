{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs rust-overlay;
in rec {
  naersk = nixpkgs.callPackage inputs.naersk {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
  rust-bin =
    (nixpkgs.appendOverlays [
      (import rust-overlay)
    ])
    .rust-bin;
  rustToolchain = rust-bin.fromRustupToolchainFile (inputs.self + "/rust-toolchain");
}
