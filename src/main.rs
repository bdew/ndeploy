use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process::ExitCode;

use crate::config::CfgObj;

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

    let config = CfgObj::load(&args.config).context("loading config")?;

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

    if !args.hosts.is_empty() {
        println!("{}", "=== Start Deploy ===".yellow().bold());
        println!();

        let futures = args.hosts.iter().map(|host| async {
            let res = commands::run_host_command(&config, &args.operation, host).await;
            (host.clone(), res)
        });

        let res = futures::future::join_all(futures).await;

        println!();
        println!("{}", "=== Result ===".yellow().bold());
        println!();

        for (host, res) in res {
            if let Err(e) = &res {
                println!("❌ {}: {}", host.bold(), format!("{e}").red());
            } else {
                println!("✅ {}: {}", host.bold(), "Success".green());
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
