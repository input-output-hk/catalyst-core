{
  description = "Voting tools for catalyst";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    flake-utils.url = "github:numtide/flake-utils";
    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { self
    , nixpkgs
    , flake-compat
    , flake-utils
    , gitignore
    , naersk
    }:

    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = (import nixpkgs) {
        inherit system;
      };

      naersk' = pkgs.callPackage naersk { };

    in
    rec {
      # For `nix build` & `nix run`:
      defaultPackage = naersk'.buildPackage {
        src = ./.;
        nativeBuildInputs = with pkgs; [
          pkg-config
          postgresql
        ];
      };

      # For `nix develop` (optional, can be skipped):
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo diesel-cli postgresql pkg-config ];
      };
    }
    );
}
