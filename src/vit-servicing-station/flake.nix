{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, utils }:
  utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
  let
    overlay = self: super: {
      vit-servicing-station = self.callPackage (
        { lib, rustPlatform, fetchFromGitHub, pkg-config, openssl, protobuf, rustfmt }:
        rustPlatform.buildRustPackage rec {
          pname = "vit-servicing-station";
          version = "HEAD";
          src = ./.;
          cargoSha256 = "sha256-U7wtPdIGuwbF2GEanVi2zUDz+gFM26ttT3OB7vQ3Ar4=";
          nativeBuildInputs = [ pkg-config protobuf rustfmt ];
          buildInputs = [ openssl ];
          configurePhase =''
            cc=$CC
          '';
          doCheck = false;
          doInstallCheck = false;
          PROTOC="${protobuf}/bin/protoc";
          PROTOC_INCLUDE="${protobuf}/include";
        }
      ) {};
    };
    pkgs = import nixpkgs { inherit system; overlays = [ overlay ]; };
  in {
    packages.vit-servicing-station = pkgs.vit-servicing-station;
    inherit overlay;
  });
}
