{
  nixConfig.extra-substituters = [
    "https://vit.cachix.org"
    "https://hydra.iohk.io"
  ];
  nixConfig.extra-trusted-public-keys = [
    "vit.cachix.org-1:tuLYwbnzbxLzQHHN0fvZI2EMpVm/+R7AKUGqukc6eh8="
    "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
  ];
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    flake-utils.url = "github:numtide/flake-utils";
    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };


  outputs =
    { self
    , nixpkgs
    , flake-compat
    , flake-utils
    , gitignore
    , rust-overlay
    , naersk
    ,
    }:
    flake-utils.lib.eachSystem
      [
        flake-utils.lib.system.x86_64-linux
        flake-utils.lib.system.aarch64-linux
      ]
      (
        system:
        let
          readTOML = file: builtins.fromTOML (builtins.readFile file);
          workspaceCargo = readTOML ./Cargo.toml;

          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          mkRust = { channel ? "stable" }:
            let
              _rust = pkgs.rust-bin.${channel}.latest.default.override {
                extensions = [
                  "rust-src"
                  "rust-analysis"
                  "rls-preview"
                  "rustfmt-preview"
                  "clippy-preview"
                ];
              };
            in
            pkgs.buildEnv {
              name = _rust.name;
              inherit (_rust) meta;
              buildInputs = [ pkgs.makeWrapper pkgs.openssl ];
              paths = [ _rust ];
              pathsToLink = [ "/" "/bin" ];
              # XXX: This is needed because cargo and clippy commands need to
              # also be aware of other binaries in order to work properly.
              # https://github.com/cachix/pre-commit-hooks.nix/issues/126
              postBuild = ''
                for i in $out/bin/*; do
                  wrapProgram "$i" --prefix PATH : "$out/bin"
                done
              '';
            };

          rust-stable = mkRust { channel = "stable"; };
          rust-nightly = mkRust { channel = "nightly"; };

          naersk-lib = naersk.lib."${system}".override {
            cargo = rust-stable;
            rustc = rust-stable;
          };

          mkPackage =
            let
              pkgCargo = readTOML ./Cargo.toml;
            in
            naersk-lib.buildPackage {
              inherit (pkgCargo.package) name version;

              root = gitignore.lib.gitignoreSource self;

              nativeBuildInputs = with pkgs; [
                pkg-config
                postgresql
              ];

            };



        in
        rec {
          packages =
            {
              default = mkPackage;
            };

          devShells.default = pkgs.mkShell {
            buildInputs =
              [ rust-stable ]
              ++ (with pkgs; [
                pkg-config
                postgresql
                diesel-cli
                cargo-insta # snapshot testing lib
              ]);
          };


          hydraJobs = packages;

          defaultPackage = packages.default;
          devShell = devShells.default;
        }
      );
}
