{
  description = "Catalyst Core";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
      flake-utils.url = "github:numtide/flake-utils";
      rust-overlay.url = "github:oxalica/rust-overlay";
      rust-overlay.inputs.flake-utils.follows = "flake-utils";
      rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: flake-utils.lib.eachDefaultSystem (system:

    let pkgs = import nixpkgs {
      inherit system;
      overlays = [ (import rust-overlay) ];
    };

    in

    {

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rust-bin.stable.latest.default

          pkg-config
          openssl
          protobuf
          uniffi-bindgen
        ];
      };

    });
}
