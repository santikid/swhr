# Simple Webhook Runner (swhr)

A lightweight, configurable webhook server that executes local scripts in response to HTTP requests.

## Usage

```bash
# Basic usage with default config file (swhr.yaml)
swhr

# Specify a custom config file
swhr -c my-config.yaml

# Use a different listen address
swhr -l 0.0.0.0:8080

# Set log level
swhr --log-level debug
```

### Command-line Options

```
Simple Webhook Runner (swhr)

Usage: swhr [OPTIONS]

Options:
  -c, --config <CONFIG>        Path to configuration file [default: swhr.yaml]
  -l, --listen <LISTEN>        Listen address in format IP:PORT [default: 127.0.0.1:3344]
      --log-level <LOG_LEVEL>  Log level (trace, debug, info, warn, error) [default: info]
  -h, --help                   Print help
  -V, --version                Print version
```

## Configuration

The configuration file is in YAML format. Here's an example:

```yaml
services:
  - path: "/service1"
    method: "POST"
    dir: "/home/user/test_d1"
    script: "/home/user/script_1"
    api_key: "ANY_UTF8_STRING" # optional

  - path: "/service2"
    method: "GET"
    dir: "/home/user/test_d2"
    script: "/home/user/script_2"
```

Webhook body is passed in `WEBHOOK_BODY` environment variable; **only UTF-8 valid Strings are supported**

```bash
echo $WEBHOOK_BODY # prints the body
```

## Using as a NixOS service

This package includes a NixOS module that allows you to easily set up SWHR as a systemd service.

### Example NixOS Configuration

```nix
# In your flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    swhr.url = "github:santikid/swhr";
  };

  outputs = { self, nixpkgs, swhr, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        swhr.nixosModule
      ];
    };
  };
}

# In your configuration.nix
{ config, pkgs, ... }:

{
  services.swhr = {
    enable = true;
    listenAddress = "0.0.0.0:3344"; # Listen on all interfaces
    logLevel = "info";
    
    # Optional: run as a different user/group
    user = "mywebhookuser";
    group = "mywebhookgroup";
    
    # Webhook configuration
    configuration = {
      services = [
        {
          path = "/webhook/deploy";
          method = "POST";
          script = "/opt/scripts/deploy.sh";
          dir = "/opt/deploy";
          api_key = "my-secret-key";
        }
        {
          path = "/webhook/backup";
          method = "GET";
          script = "/opt/scripts/backup.sh";
          dir = "/opt/backup";
        }
      ];
    };
  };
}
```
