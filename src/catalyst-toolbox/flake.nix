{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    rust-nix.url = "github:input-output-hk/rust.nix/work";
    rust-nix.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, utils, rust-nix }:

    utils.lib.simpleFlake rec {
      inherit nixpkgs;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      preOverlays = [ rust-nix overlay ];
      overlay = final: prev: {
         vit-kedqr = prev.rust-nix.buildPackage {
            inherit ((builtins.fromTOML
              (builtins.readFile (./Cargo.toml))).package)
              name version;
            root = ./.;
          };
      };
      packages =
        { catalyst-toolbox }@pkgs:
        pkgs;
      devShell = { mkShell, rustc, cargo, pkg-config, openssl }: mkShell {
        buildInputs = [ rustc cargo pkg-config openssl ];
      };
    };
}
