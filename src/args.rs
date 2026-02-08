use clap::Parser;
use clap::ValueEnum;
use clap::ValueHint;

#[derive(Debug, ValueEnum, Clone, PartialEq, Eq)]
pub enum Operation {
    Switch,
    Boot,
    Test,
    DryActivate,
    DryBuild,
}

#[derive(Parser, Debug)]
#[command(version, about = "nixos deploy utility")]
pub struct Args {
    /// Config file to use
    #[arg(short, long, default_value = "machines.yaml", value_hint = ValueHint::FilePath)]
    pub config: String,

    /// Run "nix flake update" before build and deploy
    #[arg(short, long, default_value_t = false)]
    pub update: bool,

    /// Run "nom build" to build the default package in the flake before deploying
    #[arg(short, long, default_value_t = false)]
    pub build: bool,

    /// Run on all hosts
    #[arg(short, long, default_value_t = false)]
    pub all: bool,

    /// Operation (from nixos-rebuild) to perform
    #[arg(value_enum, short, long, default_value_t = Operation::Switch)]
    pub operation: Operation,

    /// Command to execute remotely
    #[arg(short, long, value_hint = ValueHint::CommandString)]
    pub run: Option<String>,

    /// Reboot system after deployment
    #[arg(short = 'R', long, default_value_t = false)]
    pub reboot: bool,

    /// Hosts to deploy to
    #[arg(required = false)]
    pub hosts: Vec<String>,
}
