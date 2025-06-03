{
  description = "Flake for Rust & Node.js development, building a Rust project and a kiosk ISO";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    let
      # Function to create the NixOS module for a specific system
      mkNixosModule = system: { config, lib, pkgs, ... }:
        let 
          cfg = config.services.swhr;
          swhrPackage = self.packages.${system}.swhr;
          configFile = pkgs.writeText "swhr.yaml" (builtins.toJSON cfg.configuration);
        in {
          options.services.swhr = {
            enable = lib.mkEnableOption "SWHR webhook runner service";
            
            configuration = lib.mkOption {
              type = lib.types.attrs;
              default = {};
              description = "SWHR configuration (will be converted to YAML)";
              example = {
                services = [
                  {
                    path = "/webhook/example";
                    method = "POST";
                    script = "/usr/local/bin/example_script.sh";
                    dir = "/tmp";
                  }
                  {
                    path = "/webhook/secure-example";
                    method = "POST";
                    script = "/usr/local/bin/secure_example_script.sh";
                    dir = "/tmp";
                    api_key = "my-secret-api-key";
                  }
                ];
              };
            };
            
            listenAddress = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1:3344";
              description = "Address and port for SWHR to listen on";
            };
            
            logLevel = lib.mkOption {
              type = lib.types.enum [ "trace" "debug" "info" "warn" "error" ];
              default = "info";
              description = "Log level for SWHR";
            };
            
            user = lib.mkOption {
              type = lib.types.str;
              default = "swhr";
              description = "User to run SWHR as";
            };
            
            group = lib.mkOption {
              type = lib.types.str;
              default = "swhr";
              description = "Group to run SWHR as";
            };
          };
          
          config = lib.mkIf cfg.enable {
            # Create user and group if they don't exist and aren't overridden
            users.users = lib.mkIf (cfg.user == "swhr") {
              swhr = {
                isSystemUser = true;
                description = "SWHR webhook runner user";
                group = cfg.group;
              };
            };
            
            users.groups = lib.mkIf (cfg.group == "swhr") {
              swhr = {};
            };
            
            # Define the systemd service
            systemd.services.swhr = {
              description = "Simple Webhook Runner";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              
              serviceConfig = {
                ExecStart = "${swhrPackage}/bin/swhr --config ${configFile} --listen ${cfg.listenAddress} --log-level ${cfg.logLevel}";
                Restart = "on-failure";
                RestartSec = "5s";
                
                # Security settings
                User = cfg.user;
                Group = cfg.group;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                NoNewPrivileges = true;
                
                # Ensure the service has access to its configuration
                ReadWritePaths = "";
                ReadOnlyPaths = [ "${configFile}" ];
              };
            };
          };
        };
    in flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      lib = pkgs.lib;
      buildInputs = with pkgs; [
        rustc
        cargo

        rustfmt
        clippy
        rust-analyzer

        pkg-config
        openssl
        zlib
      ];
    in {
      # Development shell for Rust and Node.js (node 23 + pnpm)
      devShells.default = pkgs.mkShell {
        inherit buildInputs;
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };

      packages.swhr = pkgs.rustPlatform.buildRustPackage {
        pname = "swhr";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        cargoSha256 = lib.fakeSha256;
        buildInputs = buildInputs;
        nativeBuildInputs = buildInputs;
      };

      # NixOS module for the systemd service for this specific system
      nixosModules.default = mkNixosModule system;
    }) // {
      # Make the NixOS module available at the flake level - platform independent module
      nixosModule = { pkgs, ... }: {
        imports = [ (mkNixosModule pkgs.stdenv.hostPlatform.system) ];
      };
    };
}
