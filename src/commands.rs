use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use tokio::process::Command;

use crate::args::Operation;
use crate::config::CfgObj;
use crate::config::Host;
use crate::run_util;

static NOM_PATH: &str = match option_env!("NOM_PATH") {
    Some(path) => path,
    None => "nom",
};

pub async fn run_update(cfg: &CfgObj) -> Result<()> {
    println!("{}", "=== Start Update ===".yellow().bold());
    println!();
    let mut cmd = Command::new("nix");
    cmd.arg("flake");
    cmd.arg("update");
    cmd.arg("--flake");
    cmd.arg(cfg.flake_path.clone());
    run_util::run_command("update", cmd, false).await?;
    println!();
    Ok(())
}

pub async fn run_build(cfg: &CfgObj) -> Result<()> {
    println!("{}", "=== Start Build ===".yellow().bold());
    println!();
    let mut cmd = Command::new(NOM_PATH);
    cmd.arg("build");
    cmd.arg(cfg.flake_path.clone());
    run_util::run_command("build", cmd, false).await?;
    println!();
    Ok(())
}

pub fn operation_arg(op: &Operation) -> &'static str {
    match op {
        Operation::Switch => "switch",
        Operation::Boot => "boot",
        Operation::Test => "test",
        Operation::DryActivate => "dry-activate",
        Operation::DryBuild => "dry-build",
    }
}

pub async fn run_host_command(cfg: &CfgObj, op: &Operation, host_name: &str) -> Result<()> {
    let host = cfg.hosts.get(host_name).context("host not found")?;

    let mut cmd = Command::new("nixos-rebuild");

    cmd.arg(operation_arg(op));
    cmd.arg("--flake");
    cmd.arg(format!("{}#{}", cfg.flake_path, host_name));

    match host {
        Host::Local { _type, sudo } => {
            if !matches!(sudo, Some(false)) {
                cmd.arg("--use-remote-sudo");
            }
        }
        Host::Remote { user, addr, sudo, no_tty, substitutes } => {
            cmd.arg("--target-host");
            cmd.arg(format!("{user}@{addr}"));

            if !matches!(no_tty, Some(false)) {
                cmd.arg("--no-ssh-tty");
            }

            if !matches!(substitutes, Some(false)) {
                cmd.arg("--use-substitutes");
            }

            if matches!(sudo, Some(true)) || (sudo.is_none() && user != "root") {
                cmd.arg("--use-remote-sudo");
            }
        }
    }

    println!("{}: Running {:?}", host_name.purple().bold(), cmd.as_std());

    run_util::run_command(host_name, cmd, true).await?;

    Ok(())
}
