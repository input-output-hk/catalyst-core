{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
  };
  outputs = { self, nixpkgs, utils }:
    let
      workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      inherit (workspaceCargo.workspace) members;
    in utils.lib.simpleFlake {
      inherit nixpkgs;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      preOverlays = [ ];
      overlay = final: prev:
        let inherit (prev) lib;
        in lib.listToAttrs (lib.forEach members (member:
          lib.nameValuePair member (prev.rustPlatform.buildRustPackage {
            inherit ((builtins.fromTOML
              (builtins.readFile (./. + "/${member}/Cargo.toml"))).package)
              name version;
            src = ./.;
            cargoSha256 = "1npj9j6na14h4m052qsrd56dw8d1p4ssv0wiq1bdx2726j1wbgac";
            nativeBuildInputs = with final; [ pkg-config protobuf rustfmt ];
            buildInputs = with final; [ openssl ];
            PROTOC = "${final.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${final.protobuf}/include";
          })));
      packages = { vit-servicing-station-cli, vit-servicing-station-lib
        , vit-servicing-station-server, vit-servicing-station-tests }@pkgs:
        pkgs;
      devShell = { mkShell, rustc, cargo, pkg-config, openssl, protobuf }:
        mkShell {
          PROTOC = "${protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${protobuf}/include";
          buildInputs = [ rustc cargo pkg-config openssl protobuf ];
        };
    };
}
