use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, ValueEnum, Clone, PartialEq, Eq)]
pub enum Operation {
    Switch,
    Boot,
    Dry,
}

#[derive(Parser, Debug)]
#[command(version, about = "nixos deploy utility")]
pub struct Args {
    /// Config file to use
    #[arg(short, long, default_value = "machines.yaml")]
    pub config: String,

    /// Run flake update before build and deply
    #[arg(short, long, default_value_t = false)]
    pub update: bool,

    /// Run build before deploy
    #[arg(short, long, default_value_t = false)]
    pub build: bool,

    /// Hosts to deploy to
    #[arg(required = false)]
    pub hosts: Vec<String>,

    /// Operation to perform
    #[arg(value_enum, short, long, default_value_t = Operation::Switch)]
    pub operation: Operation,
}
