{
  nixConfig.extra-substituters = [
    "https://iog.cachix.org"
    "https://hydra.iohk.io"
  ];
  nixConfig.extra-trusted-public-keys = [
    "iog.cachix.org-1:nYO0M9xTk/s5t1Bs9asZ/Sww/1Kt/hRhkLP0Hhv/ctY="
    "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
  ];

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    cardano-node.url = "github:input-output-hk/cardano-node/1.33.0";
  };

  outputs = {
    self,
    nixpkgs,
    flake-compat,
    flake-utils,
    rust-overlay,
    naersk,
    cardano-node,
  }:
    flake-utils.lib.eachSystem
    [
      flake-utils.lib.system.x86_64-linux
      flake-utils.lib.system.x86_64-darwin
    ]
    (
      system: let
        readTOML = file: builtins.fromTOML (builtins.readFile file);
        workspaceCargo = readTOML ./Cargo.toml;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (cardano-node.packages.${system}) cardano-cli;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;

        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        mkPackage = {
          pkgPath,
          pkgCargo,
        }: let
          name = pkgCargo.package.name;
          cargoOptions =
            ["--package" "${name}"]
            ++ (pkgs.lib.optionals (name == "jormungandr") [
              "--features"
              "prometheus-metrics"
            ]);
          nativeBuildInputs = with pkgs;
            [
              pkg-config
              protobuf
              rustfmt
            ]
            ++ (pkgs.lib.optionals (name == "voting_tools_rs" || name == "vit-servicing-station-server") [
              postgresql
            ]);

          unwrapped = naersk'.buildPackage {
            inherit (pkgCargo.package) name version;
            inherit nativeBuildInputs;

            root = self;

            cargoBuildOptions = x: x ++ cargoOptions;
            cargoTestOptions = x: x ++ cargoOptions;

            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${pkgs.protobuf}/include";

            buildInputs = with pkgs; [
              openssl
            ];
          };

          extraBinPath = {
            snapshot-trigger-service = with workspace; [voting_tools_rs];
            registration-service = with workspace; [catalyst-toolbox jcli cardano-cli voting_tools_rs];
            registration-verify-service = with workspace; [jcli];
          };
        in
          if builtins.elem name (builtins.attrNames extraBinPath)
          then
            pkgs.runCommand "wrapped-${unwrapped.name}" {nativeBuildInputs = [pkgs.makeWrapper];} ''
              mkdir -p $out/bin
              ln -s ${unwrapped}/bin/${name} $out/bin/${name}
              wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.lib.makeBinPath extraBinPath.${name}}
            ''
          else unwrapped;

        workspace =
          builtins.listToAttrs
          (
            builtins.map
            (pkgPath: let
              inherit pkgPath;
              pkgCargo = readTOML ./${pkgPath}/Cargo.toml;
            in {
              name = pkgCargo.package.name;
              value = mkPackage {inherit pkgPath pkgCargo;};
            })
            workspaceCargo.workspace.members
          );

        jormungandr-entrypoint = let
          script =
            pkgs.writeShellScriptBin "entrypoint"
            ''
              set -exuo pipefail

              ulimit -n 1024

              nodeConfig="$NOMAD_TASK_DIR/node-config.json"
              runConfig="$NOMAD_TASK_DIR/running.json"
              runYaml="$NOMAD_TASK_DIR/running.yaml"
              name="jormungandr"

              chmod u+rwx -R "$NOMAD_TASK_DIR" || true

              function convert () {
                chmod u+rwx -R "$NOMAD_TASK_DIR" || true
                cp "$nodeConfig" "$runConfig"
                remarshal --if json --of yaml "$runConfig" > "$runYaml"
              }

              if [ "$RESET" = "true" ]; then
                echo "RESET is given, will start from scratch..."
                rm -rf "$STORAGE_DIR"
              elif [ -d "$STORAGE_DIR" ]; then
                echo "$STORAGE_DIR found, not restoring from backup..."
              else
                echo "$STORAGE_DIR not found, restoring backup..."

                restic restore latest \
                  --verbose=5 \
                  --no-lock \
                  --tag "$NAMESPACE" \
                  --target / \
                || echo "couldn't restore backup, continue startup procedure..."
              fi

              set +x
              echo "waiting for $REQUIRED_PEER_COUNT peers"
              until [ "$(jq -e -r '.p2p.trusted_peers | length' < "$nodeConfig" || echo 0)" -ge $REQUIRED_PEER_COUNT ]; do
                sleep 1
              done
              set -x

              convert

              if [ -n "$PRIVATE" ]; then
                echo "Running with node with secrets..."
                exec jormungandr \
                  --storage "$STORAGE_DIR" \
                  --config "$NOMAD_TASK_DIR/running.yaml" \
                  --genesis-block $NOMAD_TASK_DIR/block0.bin/block0.bin \
                  --secret $NOMAD_SECRETS_DIR/bft-secret.yaml \
                  "$@" || true
              else
                echo "Running with follower node..."
                exec jormungandr \
                  --storage "$STORAGE_DIR" \
                  --config "$NOMAD_TASK_DIR/running.yaml" \
                  --genesis-block $NOMAD_TASK_DIR/block0.bin/block0.bin \
                  "$@" || true
              fi
            '';
        in
          pkgs.symlinkJoin {
            name = "entrypoint";
            paths =
              [script workspace.jormungandr]
              ++ (with pkgs; [
                bashInteractive
                coreutils
                curl
                diffutils
                fd
                findutils
                gnugrep
                gnused
                htop
                jq
                lsof
                netcat
                procps
                remarshal
                restic
                ripgrep
                strace
                tcpdump
                tmux
                tree
                util-linux
                vim
                yq
              ]);
          };

        warnToUpdateNix = pkgs.lib.warn "Consider updating to Nix > 2.7 to remove this warning!";
      in rec {
        packages = workspace;
        devShells.default = pkgs.mkShell {
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            protobuf
            uniffi-bindgen
            postgresql
            diesel-cli
            cargo-insta # snapshot testing lib
          ];
          shellHook = ''
            echo "=== Catalyst Core development shell ==="
          '';
        };

        devShell = warnToUpdateNix devShells.default;
      }
    );
}
