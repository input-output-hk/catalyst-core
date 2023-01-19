{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  package = cell.packages.vit-servicing-station-server;
in {
  vit-servicing-station-server = std.lib.ops.mkOperable {
    inherit package;
    debugInputs = lib.containerCommonDebug;
    runtimeScript = ''
      echo ">>> Entering entrypoint script..."

      # Verify the config exists
      if [[ ! -f "$CONFIG_PATH" ]]; then
        echo "ERROR: configuration file does not exist at: $CONFIG_PATH";
        echo ">>> Aborting..."
        exit 1
      fi

      # Verify the database connection details exists
      if [[ -z "$DB_URL" ]]; then
        echo "ERROR: must supply database connection URL";
        echo ">>> Aborting..."
        exit 1
      fi

      # Allow overriding vit-servicing-station-server binary
      BIN_PATH=''${BIN_PATH:=${l.getExe package}}

      echo ">>> Using the following parameters:"
      echo "Config file: $CONFIG_PATH"

      args+=()
      args+=("--in-settings-file" "$CONFIG_PATH")
      args+=("--service-version" "$VERSION")
      args+=("--db-url" "$DB_URL")

      echo ">>> Running servicing station..."
      exec "$BIN_PATH" "''${args[@]}"
    '';
  };
}
