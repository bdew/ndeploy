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

static NIXOS_REBUILD_PATH: &str = match option_env!("NIXOS_REBUILD_PATH") {
    Some(path) => path,
    None => "nixos-rebuild",
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

pub async fn run_deploy(
    config: &CfgObj,
    op: &Operation,
    hosts: &[String],
    reboot: bool,
) -> Result<()> {
    println!("{}", "=== Start Deploy ===".yellow().bold());
    println!();

    let futures = hosts.iter().map(|host| async {
        let res = run_host_deploy(config, op, host, reboot).await;
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

    Ok(())
}

pub async fn run_host_deploy(
    cfg: &CfgObj,
    op: &Operation,
    host_name: &str,
    reboot: bool,
) -> Result<()> {
    let host = cfg.hosts.get(host_name).context("host not found")?;

    let mut cmd = Command::new(NIXOS_REBUILD_PATH);

    cmd.arg(operation_arg(op));
    cmd.arg("--flake");
    cmd.arg(format!("{}#{}", cfg.flake_path, host_name));
    cmd.arg("--no-reexec");

    match host {
        Host::Local { _type, sudo } => {
            if !matches!(sudo, Some(false)) {
                cmd.arg("--sudo");
            }
        }
        Host::Remote {
            user,
            addr,
            sudo,
            substitutes,
        } => {
            cmd.arg("--target-host");
            cmd.arg(format!("{user}@{addr}"));

            if !matches!(substitutes, Some(false)) {
                cmd.arg("--use-substitutes");
            }

            if matches!(sudo, Some(true)) || (sudo.is_none() && user != "root") {
                cmd.arg("--sudo");
            }
        }
    }

    println!("{}: Running {:?}", host_name.purple().bold(), cmd.as_std());

    run_util::run_command(host_name, cmd, true).await?;

    if reboot {
        run_host_reboot(cfg, host_name).await?;
    }

    Ok(())
}

pub async fn run_host_reboot(cfg: &CfgObj, host_name: &str) -> Result<()> {
    let host = cfg.hosts.get(host_name).context("host not found")?;

    match host {
        Host::Local { _type: _, sudo: _ } => {
            println!(
                "{}: Skipping reboot for local host (reboot manually if needed)",
                host_name.purple().bold()
            );
        }
        Host::Remote {
            user,
            addr,
            sudo,
            substitutes: _,
        } => {
            println!("{}: Rebooting system...", host_name.purple().bold());

            let mut ssh_cmd = Command::new("ssh");
            ssh_cmd.arg(format!("{user}@{addr}"));
            ssh_cmd.arg("-T");

            let reboot_command =
                if matches!(sudo, Some(true)) || (sudo.is_none() && user != "root") {
                    "sudo reboot"
                } else {
                    "reboot"
                };

            ssh_cmd.arg(reboot_command);

            run_util::run_command(host_name, ssh_cmd, true).await?;

            println!(
                "{}: Waiting 30 seconds for system to reboot...",
                host_name.purple().bold()
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            // Try to connect and run nixos-version with retries
            for attempt in 1..=10 {
                let mut check_cmd = Command::new("ssh");
                check_cmd.arg(format!("{user}@{addr}"));
                check_cmd.arg("-T");
                check_cmd.arg("-o");
                check_cmd.arg("ConnectTimeout=5");
                check_cmd.arg("nixos-version");

                match run_util::run_command(host_name, check_cmd, true).await {
                    Ok(_) => {
                        println!("{}: System is back up!", host_name.purple().bold());
                        break;
                    }
                    Err(_) if attempt < 10 => {
                        println!(
                            "{}: Connection attempt {} failed, retrying in 10 seconds...",
                            host_name.purple().bold(),
                            attempt
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                    Err(e) => {
                        anyhow::bail!(
                            "Failed to connect to host after reboot (10 attempts): {}",
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn run_command(config: &CfgObj, hosts: &[String], cmd: &str) -> Result<()> {
    println!("{}", format!("=== Start Run: {cmd} ===").yellow().bold());
    println!();

    let futures = hosts.iter().map(|host| async {
        let res = run_host_command(config, host, cmd).await;
        if let Err(e) = &res {
            println!("{}: ❌ {}", host.red().bold(), format!("{e}").red());
        };
    });

    futures::future::join_all(futures).await;

    Ok(())
}

pub async fn run_host_command(cfg: &CfgObj, host_name: &str, cmd_arg: &str) -> Result<()> {
    let host = cfg.hosts.get(host_name).context("host not found")?;

    match host {
        Host::Local { _type, sudo: _ } => {
            println!("{}: Skipping local host", host_name.purple().bold());
        }
        Host::Remote {
            user,
            addr,
            sudo: _,
            substitutes: _,
        } => {
            let mut cmd = Command::new("ssh");

            cmd.arg(format!("{user}@{addr}"));

            cmd.arg("-T");
            cmd.arg(cmd_arg);

            run_util::run_command(host_name, cmd, true).await?;
        }
    }

    Ok(())
}
