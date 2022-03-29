{
  description = "Incubator for catalyst related testing projects";

  nixConfig.extra-substituters = [
    "https://hydra.iohk.io"
    "https://vit.cachix.org"
  ];
  nixConfig.extra-trusted-public-keys = [
    "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
    "vit.cachix.org-1:tuLYwbnzbxLzQHHN0fvZI2EMpVm/+R7AKUGqukc6eh8="
  ];

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.gitignore.url = "github:hercules-ci/gitignore.nix";
  inputs.gitignore.inputs.nixpkgs.follows = "nixpkgs";
  inputs.pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  inputs.pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  inputs.pre-commit-hooks.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  # XXX: https://github.com/nix-community/naersk/pull/167
  #inputs.naersk.url = "github:nix-community/naersk";
  inputs.naersk.url = "github:yusdacra/naersk/feat/cargolock-git-deps";
  inputs.naersk.inputs.nixpkgs.follows = "nixpkgs";
  inputs.voting-tools.url = "github:input-output-hk/voting-tools?rev=6da7c45cbd1c756285ca2a1db99f82dd1a8cc16b";
  inputs.vit-kedqr.url = "github:input-output-hk/vit-kedqr";
  inputs.vit-servicing-station.url = "github:input-output-hk/vit-servicing-station/master";
  inputs.jormungandr_.url = "github:input-output-hk/jormungandr/master";
  inputs.cardano-node.url = "github:input-output-hk/cardano-node/1.33.0";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    gitignore,
    pre-commit-hooks,
    rust-overlay,
    naersk,
    voting-tools,
    vit-kedqr,
    vit-servicing-station,
    jormungandr_,
    cardano-node,
  }:
    flake-utils.lib.eachSystem
    [
      flake-utils.lib.system.x86_64-linux
      flake-utils.lib.system.aarch64-linux
    ]
    (
      system: let
        readTOML = file: builtins.fromTOML (builtins.readFile file);
        workspaceCargo = readTOML ./Cargo.toml;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (voting-tools.packages.${system}) voting-tools voter-registration;
        inherit (jormungandr_.packages.${system}) jormungandr jcli;
        inherit (vit-servicing-station.legacyPackages.${system}) vit-servicing-station-server;
        inherit (cardano-node.packages.${system}) cardano-cli;

        rust = let
          _rust = pkgs.rust-bin.stable.latest.default.override {
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
            buildInputs = [pkgs.makeWrapper];
            paths = [_rust];
            pathsToLink = ["/" "/bin"];
            # XXX: This is needed because cargo and clippy commands need to
            # also be aware of other binaries in order to work properly.
            # https://github.com/cachix/pre-commit-hooks.nix/issues/126
            postBuild = ''
              for i in $out/bin/*; do
                wrapProgram "$i" --prefix PATH : "$out/bin"
              done
            '';
          };

        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        mkPackage = name: let
          pkgCargo = readTOML ./${name}/Cargo.toml;
          cargoOptions = ["--package" name];
        in
          naersk-lib.buildPackage {
            root = gitignore.lib.gitignoreSource self;

            cargoBuildOptions = x: x ++ cargoOptions;
            cargoTestOptions = x: x ++ cargoOptions;

            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${pkgs.protobuf}/include";

            nativeBuildInputs = with pkgs;
              [
                pkg-config
                protobuf
                rustfmt
              ]
              ++ (pkgs.lib.optional
                (builtins.elem name [
                  "snapshot-trigger-service"
                  "registration-service"
                  "registration-verify-service"
                ])
                pkgs.makeWrapper);

            buildInputs = with pkgs; [
              openssl
            ];

            postInstall =
              if name == "snapshot-trigger-service"
              then "wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.lib.makeBinPath [voting-tools]}"
              else if name == "registration-service"
              then "wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.lib.makeBinPath [vit-kedqr jcli cardano-cli]}"
              else if name == "registration-verify-service"
              then "wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.lib.makeBinPath [jcli]}"
              else "";
          };

        workspace =
          builtins.listToAttrs
          (
            builtins.map
            (name: {
              inherit name;
              value = mkPackage name;
            })
            workspaceCargo.workspace.members
          );

        pre-commit = pre-commit-hooks.lib.${system}.run {
          src = self;
          hooks = {
            alejandra = {
              enable = true;
            };
            rustfmt = {
              enable = true;
              entry = pkgs.lib.mkForce "${rust}/bin/cargo-fmt fmt -- --check --color always";
            };
          };
        };

        warnToUpdateNix = pkgs.lib.warn "Consider updating to Nix > 2.7 to remove this warning!";
      in rec {
        packages = {
          inherit
            (workspace)
            iapyx
            vitup
            integration-tests
            snapshot-trigger-service
            registration-service
            registration-verify-service
            ;
          inherit voting-tools;
          default = workspace.vitup;
        };

        devShells.default = pkgs.mkShell {
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          buildInputs =
            [rust]
            ++ (with pkgs; [
              pkg-config
              openssl
              protobuf
            ]);
          shellHook =
            pre-commit.shellHook
            + ''
              export PATH="${jormungandr}/bin:$PATH"
              export PATH="${vit-servicing-station-server}/bin:$PATH"
              echo "=== Development shell ==="
              echo "Info: Git hooks can be installed using \`pre-commit install\`"
            '';
        };

        checks.pre-commit = pre-commit;

        hydraJobs = packages;

        defaultPackage = warnToUpdateNix packages.default;
        devShell = warnToUpdateNix devShells.default;
      }
    );
}
