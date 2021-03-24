{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    rust-nix.url = "github:input-output-hk/rust.nix/work";
    rust-nix.inputs.nixpkgs.follows = "nixpkgs";
    voting-tools.url = "github:input-output-hk/voting-tools";
  };
  outputs = { self, nixpkgs, utils, rust-nix, voting-tools }:
    let
      workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      inherit (workspaceCargo.workspace) members;
    in utils.lib.simpleFlake {
      inherit nixpkgs;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      preOverlays = let
        cargo-packages = final: prev:
        let lib = prev.lib;
        in lib.listToAttrs (lib.forEach members (member:
          lib.nameValuePair member (final.rust-nix.buildPackage {
            inherit ((builtins.fromTOML
              (builtins.readFile (./. + "/${member}/Cargo.toml"))).package)
              name version;
            root = ./.;
            nativeBuildInputs = with final; [ pkg-config protobuf rustfmt ];
            buildInputs = with final; [ openssl ];
            PROTOC = "${final.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${final.protobuf}/include";
          })));
      in [ rust-nix cargo-packages ];
      overlay = final: prev: {
        inherit (voting-tools.packages.${final.system}) voting-tools;
        snapshot-trigger-service = prev.snapshot-trigger-service.overrideAttrs (oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ final.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/snapshot-trigger-service --prefix PATH : ${
              final.lib.makeBinPath [ final.voting-tools ]
            }
          '';
        });
      };
      packages =
        { iapyx, vitup, integration-tests, snapshot-trigger-service }@pkgs:
        pkgs;
      devShell = { mkShell, rustc, cargo, pkg-config, openssl, protobuf }: mkShell {
        PROTOC = "${protobuf}/bin/protoc";
        PROTOC_INCLUDE = "${protobuf}/include";
        buildInputs = [ rustc cargo pkg-config openssl protobuf ];
      };
    };
}
