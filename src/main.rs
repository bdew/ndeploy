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
    let args = Box::leak(Box::new(args::Args::parse()));

    if (!args.build && !args.update) && args.hosts.is_empty() {
        eprintln!("Error: No hosts specified and neither build nor update is requested.");
        return Ok(ExitCode::FAILURE);
    }

    let config = Box::leak(Box::new(
        CfgObj::load(&args.config).context("loading config")?,
    ));

    for host in &args.hosts {
        if !config.hosts.contains_key(host) {
            eprintln!("Error: Host '{host}' not found in config.");
            return Ok(ExitCode::FAILURE);
        }
    }

    if args.update {
        commands::run_update(config).await?;
    }

    if args.build {
        commands::run_build(config).await?;
    }

    if !args.hosts.is_empty() {
        println!("{}", "=== Start Deploy ===".yellow().bold());
        println!();

        let mut join_set = tokio::task::JoinSet::new();
        for host in &args.hosts {
            join_set.spawn(async {
                let res = commands::run_host_command(config, &args.operation, host).await;
                (host.clone(), res)
            });
        }

        let res = join_set.join_all().await;

        println!();
        println!("{}", "=== Result ===".yellow().bold());
        println!();
        for res in res {
            match res {
                (host, Ok(())) => {
                    println!("{}: {}", host.bold(), "Success".green());
                }
                (host, Err(e)) => {
                    eprintln!("{}: {}", host.bold(), format!("{e}").red());
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
