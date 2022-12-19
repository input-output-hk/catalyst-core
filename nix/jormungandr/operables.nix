{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  package = cell.packages.jormungandr;

  mkOperable = namespace: let
    artifacts' = artifacts."artifacts-${namespace}";
  in
    std.lib.ops.mkOperable {
      inherit package;
      # TODO: Remove all the bitte stuff
      runtimeScript = ''
        ulimit -n 1024
        nodeConfig="$NOMAD_TASK_DIR/node-config.json"
        runConfig="$NOMAD_TASK_DIR/running.json"
        runYaml="$NOMAD_TASK_DIR/running.yaml"
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
        until [ "$(jq -e -r '.p2p.trusted_peers | length' < "$nodeConfig" || echo 0)" -ge "$REQUIRED_PEER_COUNT" ]; do
          sleep 1
        done
        set -x
        convert
        if [ -n "$PRIVATE" ]; then
          echo "Running with node with secrets..."
          exec ${l.getExe package} \
            --storage "$STORAGE_DIR" \
            --config "$NOMAD_TASK_DIR/running.yaml" \
            --genesis-block "${artifacts'}/block0.bin" \
            --secret "$NOMAD_SECRETS_DIR/bft-secret.yaml" \
            "$@" || true
        else
          echo "Running with follower node..."
          exec ${l.getExe package} \
            --storage "$STORAGE_DIR" \
            --config "$NOMAD_TASK_DIR/running.yaml" \
            --genesis-block "${artifacts'}/block0.bin" \
            "$@" || true
        fi
      '';
    };
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOperable
