{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.artifacts) artifacts;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  package = cell.packages.vit-servicing-station-server;

  mkVitOperable = namespace: let
    # Deployment artifacts, including block0
    artifacts' = artifacts."artifacts-${namespace}";

    # Configuration file contents
    config = l.toJSON {
      tls = {
        cert_file = null;
        priv_key_file = null;
      };
      cors = {
        max_age_secs = null;
        allowed_origins = [
          "https://${namespace}-servicing-station.vit.iohk.io"
          "http://127.0.0.1"
        ];
      };
      block0_path = "${artifacts'}/block0.bin";
      enable_api_tokens = false;
      log = {
        log_level = "trace";
      };
      address = "0.0.0.0:8080";
      service_version = "";
      db_url = "";
    };

    diesel-cli = nixpkgs.diesel-cli.override {
      sqliteSupport = false;
      mysqlSupport = false;
    };
  in
    std.lib.ops.mkOperable {
      inherit package;
      runtimeInputs = [
        artifacts'
        diesel-cli
      ];
      runtimeScript = let
        configFile =
          nixpkgs.runCommand "vit-ss-config-${namespace}"
          {
            inherit config;
            passAsFile = ["config"];
          }
          ''
            cp $configPath $out
          '';
      in ''
        echo ">>> Entering entrypoint script..."

        if [[ -n "$MIGRATE" ]]; then
          echo ">>> Running migrations..."
          export DATABASE_URL="$DB_URL"
          diesel migration run --migration-dir "${package}/migrations"
        fi

        echo ">>> Running servicing station..."
        exec ${l.getExe package} \
          --in-settings-file "${configFile}" \
          --service-version "$VERSION" \
          --db-url "$DB_URL"
      '';
    };
in
  {}
  // lib.mapToNamespaces "vit-servicing-station-server" mkVitOperable
