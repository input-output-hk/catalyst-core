{
  inputs = { utils.url = "github:numtide/flake-utils"; };
  outputs = { self, nixpkgs, utils }:
    utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        overlay = self: super: {
          iapyx = self.callPackage ({ lib, rustPlatform, fetchFromGitHub
            , pkg-config, openssl, protobuf, rustfmt }:
            rustPlatform.buildRustPackage rec {
              pname = "iapyx";
              version = "HEAD";
              src = ./.;
              cargoSha256 =
                "sha256-ySW8Wdq72JduCzTXS0mw8z6Dok4C4pJOrLjR2+3apls=";
              nativeBuildInputs = [ pkg-config protobuf rustfmt ];
              buildInputs = [ openssl ];
              configurePhase = ''
                cc=$CC
              '';
              doCheck = false;
              doInstallCheck = false;
              PROTOC = "${protobuf}/bin/protoc";
              PROTOC_INCLUDE = "${protobuf}/include";
            }) { };
        };
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };
      in {
        packages.iapyx = pkgs.iapyx;
        defaultPackage = pkgs.iapyx;
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [ cargo pkgconfig openssl ];
        };
        inherit overlay;
      });
}
