{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    rust-nix.url = "github:input-output-hk/rust.nix/work";
    rust-nix.inputs.nixpkgs.follows = "nixpkgs";
    voting-tools.url =
      "github:input-output-hk/voting-tools?rev=6da7c45cbd1c756285ca2a1db99f82dd1a8cc16b";
    vit-kedqr.url = "github:input-output-hk/vit-kedqr";
    vit-servicing-station.url = "github:input-output-hk/vit-servicing-station/master";
    jormungandr.url = "github:input-output-hk/jormungandr/master";
    cardano-node.url = "github:input-output-hk/cardano-node/1.33.0";
  };
  outputs = { self
            , nixpkgs
            , utils
            , rust-nix
            , voting-tools
            , vit-kedqr
            , vit-servicing-station
            , jormungandr
            , cardano-node
            }:
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

        registration-verify-service = prev.registration-verify-service.overrideAttrs
          (oldAttrs: {
            nativeBuildInputs = oldAttrs.nativeBuildInputs
              ++ [ final.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/registration-verify-service --prefix PATH : ${
                final.lib.makeBinPath [
                  jormungandr.packages.${final.system}.jcli
                ]
              }
            '';
          });
      };

      packages = { iapyx, vitup, integration-tests, snapshot-trigger-service
        , registration-service, registration-verify-service }@pkgs:
        pkgs;

      devShell = { system, mkShell, rustc, cargo, pkg-config, openssl, protobuf, rustfmt }:
        mkShell {
          PROTOC = "${protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${protobuf}/include";
          buildInputs = [ rustc cargo pkg-config openssl protobuf rustfmt ];
          shellHook = ''
            export PATH="${jormungandr.packages.${system}.jormungandr}/bin:$PATH"
            export PATH="${vit-servicing-station.legacyPackages.${system}.vit-servicing-station-server}/bin:$PATH"
          '';
        };

      hydraJobs = { iapyx, vitup, integration-tests, snapshot-trigger-service
        , registration-service, registration-verify-service }@pkgs:
        pkgs;
    };
}
