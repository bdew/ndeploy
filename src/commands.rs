use anyhow::Result;
use colored::Colorize;
use tokio::process::Command;

use crate::config::CfgObj;
use crate::run_util;

pub async fn run_update(cfg: &CfgObj) -> Result<()> {
    println!("{}", "=== Start Update ===".yellow().bold());
    println!();
    let mut cmd = Command::new("nix");
    cmd.arg("flake");
    cmd.arg("update");
    cmd.arg("--flake");
    cmd.arg(cfg.flake_path.clone());
    run_util::run_command("update", cmd, false).await?;
    Ok(())
}

pub async fn run_build(cfg: &CfgObj) -> Result<()> {
    println!("{}", "=== Start Build ===".yellow().bold());
    println!();
    let mut cmd = Command::new("nom");
    cmd.arg("build");
    cmd.arg(cfg.flake_path.clone());
    run_util::run_command("build", cmd, false).await?;
    println!();
    Ok(())
}
