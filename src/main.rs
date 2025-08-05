use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

use crate::run_util::run_command;

mod config;
mod run_util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = config::CfgObj::load("config.yaml")?;
    println!("{config:#?}");

    let flake_path =
        Path::canonicalize(Path::new(&config.flake_path)).context("Error getting flake path")?;

    println!("Flake path: {flake_path:?}");

    let mut cmd = Command::new("nix");
    cmd.arg("flake");
    cmd.arg("update");
    cmd.arg("--flake");
    cmd.arg(flake_path.clone());

    run_command("update", cmd).await?;

    let mut cmd = Command::new("nom");
    cmd.arg("build");
    cmd.arg(flake_path);

    run_command("build", cmd).await?;

    Ok(())
}
