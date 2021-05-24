{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    rust-nix.url = "github:input-output-hk/rust.nix/work";
    rust-nix.inputs.nixpkgs.follows = "nixpkgs";
    voting-tools.url =
      "github:input-output-hk/voting-tools?rev=db0748e7c1636ac21f322c9d42ba088bb501e5a2";
    vit-kedqr.url = "github:input-output-hk/vit-kedqr";
    jormungandr.url =
      "github:input-output-hk/jormungandr?rev=50fc937159cbea328973af2a4a04d1c8d9d4b48e";
    cardano-node.url =
      "github:input-output-hk/cardano-node/1.26.1-with-cardano-cli";
  };
  outputs = { self, nixpkgs, utils, rust-nix, voting-tools, vit-kedqr
    , jormungandr, cardano-node }:
    let
      workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      inherit (workspaceCargo.workspace) members;
    in utils.lib.simpleFlake {
      inherit nixpkgs;

      systems = [ "x86_64-linux" ];

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

        snapshot-trigger-service = prev.snapshot-trigger-service.overrideAttrs
          (oldAttrs: {
            nativeBuildInputs = oldAttrs.nativeBuildInputs
              ++ [ final.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/snapshot-trigger-service --prefix PATH : ${
                final.lib.makeBinPath [ final.voting-tools ]
              }
            '';
          });

        registration-service = prev.registration-service.overrideAttrs
          (oldAttrs: {
            nativeBuildInputs = oldAttrs.nativeBuildInputs
              ++ [ final.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/registration-service --prefix PATH : ${
                final.lib.makeBinPath [
                  voting-tools.packages.${final.system}.voter-registration
                  vit-kedqr.packages.${final.system}.vit-kedqr
                  jormungandr.packages.${final.system}.jcli
                  cardano-node.packages.${final.system}.cardano-cli
                ]
              }
            '';
          });
      };

      packages = { iapyx, vitup, integration-tests, snapshot-trigger-service
        , registration-service }@pkgs:
        pkgs;

      devShell = { mkShell, rustc, cargo, pkg-config, openssl, protobuf, rustfmt }:
        mkShell {
          PROTOC = "${protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${protobuf}/include";
          buildInputs = [ rustc cargo pkg-config openssl protobuf rustfmt ];
        };

      hydraJobs = { iapyx, vitup, integration-tests, snapshot-trigger-service
        , registration-service }@pkgs:
        pkgs;
    };
}
