use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

mod args;
mod commands;
mod config;
mod run_util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<ExitCode> {
    let args = args::Args::parse();

    if (!args.build && !args.update) && args.hosts.is_empty() {
        eprintln!("Error: No hosts specified and neither build nor update is requested.");
        return Ok(ExitCode::FAILURE);
    }

    let config = config::CfgObj::load(args.config).context("loading config")?;

    for host in &args.hosts {
        if !config.hosts.contains_key(host) {
            eprintln!("Error: Host '{host}' not found in config.");
            return Ok(ExitCode::FAILURE);
        }
    }

    if args.update {
        commands::run_update(&config).await?;
    }

    if args.build {
        commands::run_build(&config).await?;
    }

    Ok(ExitCode::SUCCESS)
}
