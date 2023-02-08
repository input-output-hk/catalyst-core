{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  package = cell.packages.jormungandr;
in {
  jormungandr = std.lib.ops.mkOperable {
    inherit package;
    debugInputs = lib.containerCommonDebug;
    runtimeInputs = with nixpkgs; [
      busybox # Includes nslookup
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

      # Verify genesis block is present
      if [[ ! -f "$GENESIS_PATH" ]]; then
        echo "ERROR: genesis block is absent at: $GENESIS_PATH"
        echo ">>> Aborting..."
        exit 1
      fi

      # Allow overriding jormungandr binary
      BIN_PATH=''${BIN_PATH:=${l.getExe package}}

      echo ">>> Using the following parameters:"
      echo "Storage path: $STORAGE_PATH"
      echo "Node config: $NODE_CONFIG_PATH"
      echo "Genesis block: $GENESIS_PATH"
      echo "Binary path: $BIN_PATH"

      args+=()
      args+=("--storage" "$STORAGE_PATH")
      args+=("--config" "$NODE_CONFIG_PATH")
      args+=("--genesis-block" "$GENESIS_PATH")

      if [[ -n "''${LEADER:=}" ]]; then
        echo ">>> Configuring node as leader..."

        # shellcheck disable=SC2153
        if [[ ! -f "$BFT_PATH" ]]; then
          echo "ERROR: BFT is absent at: $BFT_PATH"
          echo ">>> Aborting..."
          exit 1
        fi

        echo ">>> Using BFT at: $BFT_PATH"
        args+=("--secret" "$BFT_PATH")
      fi

      # Nodes will fail to start if they cannot resolve the domain names of
      # their respective peers. If domains are used for peers, it's necessary
      # to wait for them to resolve first before starting the node.
      if [[ -n "''${DNS_PEERS:=}" ]]; then
        for PEER in $DNS_PEERS
        do
          while ! nslookup "$PEER"; do
              echo ">>> Waiting for $PEER to be resolvable..."
              sleep 1
          done
          echo "Successfully resolved $PEER"
        done
      fi

      exec "$BIN_PATH" "''${args[@]}"
    '';
  };
}
