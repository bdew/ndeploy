use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::BufReader;
use tokio::process::Command;

pub fn wrap_stream(name: &str, err: bool, reader: impl AsyncRead + Unpin + Send + 'static) {
    let mut reader = BufReader::new(reader).lines();
    let name_prefix = if err { name.red().bold() } else { name.bold() };
    tokio::spawn(async move {
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("{name_prefix}: {line}");
        }
    });
}

pub async fn run_command(name_ref: impl AsRef<str>, mut cmd: Command, wrap: bool) -> Result<()> {
    let name = name_ref.as_ref();

    if wrap {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    cmd.stdin(Stdio::null());

    let mut child = cmd
        .spawn()
        .context(format!("Failed to run {:?}", cmd.as_std()))?;

    if wrap {
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture stdout of child process")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture stderr of child process")?;

        wrap_stream(name, false, stdout);
        wrap_stream(name, true, stderr);
    }

    let res = child
        .wait()
        .await
        .context(format!("Failed to wait for {name}"))?;

    if !res.success() {
        anyhow::bail!(
            "Command failed with exit code {}",
            res.code().unwrap_or(-1)
        );
    }

    Ok(())
}
