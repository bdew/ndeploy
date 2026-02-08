# ndeploy

`ndeploy` is a utility for deploying to multiple NixOS hosts with optional build and update steps.

## Usage

```
Usage: ndeploy [OPTIONS] [HOSTS]...

Arguments:
  [HOSTS]...  Hosts to deploy to

Options:
  -c, --config <CONFIG>        Config file to use [default: machines.yaml]
  -u, --update                 Run "nix flake update" before build and deploy
  -b, --build                  Run "nom build" to build the default package in the flake before deploying
  -a, --all                    Run on all hosts
  -o, --operation <OPERATION>  Operation (from nixos-rebuild) to perform [default: switch] [possible values: switch, boot, test, dry-activate, dry-build]
  -R, --reboot                 Reboot system after deployment
  -r, --run <RUN>              Command to execute remotely
  -h, --help                   Print help
  -V, --version                Print version
```

## Configuration File

The configuration file is a YAML file that defines available hosts and their settings. Example:

```yaml
flakePath: /some/path               # Defaults to current dir
hosts:                              # Hosts that can be deployed to
  foo:                              # Each host should match an entry in nixosConfigurations
    addr: foo.example.com
    user: somebody                  # Will use ssh somebody@foo.example.com
    sudo: true                      # Optional, adds --sudo to nixos-rebuild (default: true if user != root)
    substitutes: true               # Optional, adds --use-substitutes to nixos-rebuild (default: true)
  self:                             
    type: local                     # Will deploy to local machine
```

## Features

- If multiple hosts are passed - will perform the deploy in parallel
- Can run ssh commands on remote hosts in parallel
