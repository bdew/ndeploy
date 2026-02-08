use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

use crate::args::Operation;
use crate::config::CfgObj;

mod args;
mod commands;
mod config;
mod run_util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<ExitCode> {
    let args = args::Args::parse();

    if !args.build && !args.update && args.hosts.is_empty() && !args.all {
        eprintln!("Error: No hosts specified and neither build nor update is requested.");
        return Ok(ExitCode::FAILURE);
    }

    if !args.hosts.is_empty() && args.all {
        eprintln!("Error: Hosts can't be specified when using --all.");
        return Ok(ExitCode::FAILURE);
    }

    if args.operation != Operation::Switch && args.operation != Operation::Boot && args.reboot {
        eprintln!("Error: Reboot shouldn't be used when operation isn't switch/boot");
        return Ok(ExitCode::FAILURE);
    }

    if args.run.is_some() && args.reboot {
        eprintln!("Error: Reboot can't be used when running command");
        return Ok(ExitCode::FAILURE);
    }

    let config = CfgObj::load(&args.config).context("loading config")?;

    for host in &args.hosts {
        if !config.hosts.contains_key(host) {
            eprintln!("Error: Host '{host}' not found in config.");
            return Ok(ExitCode::FAILURE);
        }
    }

    let hosts = if args.all {
        config.hosts.keys().cloned().collect()
    } else {
        args.hosts
    };

    if args.update {
        commands::run_update(&config).await?;
    }

    if args.build {
        commands::run_build(&config).await?;
    }

    if !hosts.is_empty() {
        if let Some(cmd) = args.run {
            commands::run_command(&config, &hosts, &cmd).await?;
        } else {
            commands::run_deploy(&config, &args.operation, &hosts, args.reboot).await?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
