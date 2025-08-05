use anyhow::Context;
use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::BufReader;
use tokio::process::Command;

pub fn print_stream_lines(name: String, reader: impl AsyncRead + Unpin + Send + 'static) {
    let mut reader = BufReader::new(reader).lines();
    tokio::spawn(async move {
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("{name}: {line}");
        }
        println!("=== {name} closed ===")
    });
}

pub async fn run_command(name_ref: impl AsRef<str>, mut cmd: Command) -> Result<()> {
    let name = name_ref.as_ref();

    println!("=== Start: {name} ===");

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context(format!("Failed to run {:?}", cmd.as_std()))?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture stdout of child process")?;
    let stderr = child
        .stderr
        .take()
        .context("Failed to capture stderr of child process")?;

    print_stream_lines(format!("{name} stdout"), stdout);
    print_stream_lines(format!("{name} stderr"), stderr);

    let res = child
        .wait()
        .await
        .context(format!("Failed to wait for {name}"))?;

    if !res.success() {
        anyhow::bail!(
            "Command {name} failed with exit code {}",
            res.code().unwrap_or(-1)
        );
    }

    Ok(())
}
