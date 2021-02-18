{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    voting-tools.url = "github:input-output-hk/voting-tools";
  };
  outputs = { self, nixpkgs, utils, voting-tools }:
    let
      workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      inherit (workspaceCargo.workspace) members;
    in utils.lib.simpleFlake {
      inherit nixpkgs;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      overlay = final: prev:
        let
          lib = prev.lib;
          workspace = lib.listToAttrs (lib.forEach members (member:
            lib.nameValuePair member (final.rustPlatform.buildRustPackage (let
              inherit ((builtins.fromTOML
                (builtins.readFile (./. + "/${member}/Cargo.toml"))).package)
                name version;
            in {
              pname = name;
              version = version;
              src = builtins.filterSource (name: type:
                let baseName = builtins.baseNameOf (builtins.toString name);
                in !(baseName == ".gitignore"
                  || (type == "directory" && baseName == ".git")
                  || (type == "directory" && baseName == "target")
                  || (type == "symlink" && lib.hasPrefix "result" baseName)
                  || (lib.hasSuffix ".nix" baseName))) ./.;
              cargoSha256 =
                "sha256-LUOoqImvBX37a/GvSqSNaR0ei9+EicuKT1pcycaVqLQ=";
              nativeBuildInputs = with final; [
                pkg-config
                protobuf
                rustfmt
                makeWrapper
              ];
              buildInputs = with final; [ openssl ];
              configurePhase = ''
                cc=$CC
              '';
              postInstall = ''
                wrapProgram $out/bin/snapshot-trigger-service --prefix PATH : ${
                  lib.makeBinPath [ final.voting-tools ]
                }
              '';
              doCheck = false;
              doInstallCheck = false;
              PROTOC = "${final.protobuf}/bin/protoc";
              PROTOC_INCLUDE = "${final.protobuf}/include";
            }))));
        in workspace // {
          inherit (voting-tools.packages.${final.system}) voting-tools;
        };

      packages =
        { iapyx, vitup, integration-tests, snapshot-trigger-service }@pkgs:
        pkgs;

      devShell = { mkShell, rustc, cargo, pkg-config, openssl, protobuf }:
        mkShell {
          PROTOC = "${protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${protobuf}/include";
          buildInputs = [ rustc cargo pkg-config openssl protobuf ];
        };
    };
}
