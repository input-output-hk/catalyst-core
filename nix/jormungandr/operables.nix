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
      debugInputs = lib.containerCommonDebug;
      runtimeInputs = with nixpkgs; [
        remarshal
      ];
      runtimeScript = ''
        echo ">>> Entering entrypoint script..."

        # Verify the storage path exists
        if [[ ! -d "$STORAGE_PATH" ]]; then
          echo "ERROR: storage path does not exist at: $STORAGE_PATH";
          echo ">>> Aborting..."
          exit 1
        fi

        # Verify config is present
        if [[ ! -f "$NODE_CONFIG_PATH" ]]; then
          echo "ERROR: node configuration is absent at: $NODE_CONFIG_PATH"
          echo ">>> Aborting..."
          exit 1
        fi

        echo ">>> Using the following parameters:"
        echo "Storage path: $STORAGE_PATH"
        echo "Node config: $NODE_CONFIG_PATH"
        echo "Genesis block: ${artifacts'}/block0.bin"

        args+=()
        args+=("--storage" "$STORAGE_PATH")
        args+=("--config" "$NODE_CONFIG_PATH")
        args+=("--genesis-block" "${artifacts'}/block0.bin")

        if [[ -n "''${LEADER:=}" ]]; then
          echo ">>> Configuring node as leader..."
          if [[ ! -f "$BFT_PATH" ]]; then
            echo "ERROR: BFT is absent at: $BFT_PATH"
            echo ">>> Aborting..."
            exit 1
          fi

          echo ">>> Using BFT at: $BFT_PATH"
          args+=("--secret" "$BFT_PATH")
        fi

        exec ${l.getExe package} "''${args[@]}"
      '';
    };
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOperable
