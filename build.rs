use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::generate_to;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

include!("src/args.rs");

fn main() -> Result<(), Error> {
    let completions_dir = PathBuf::from("target/completions");

    if !fs::exists(&completions_dir)? {
        fs::create_dir(&completions_dir)?;
    }

    let mut cmd = Args::command();
    let name = cmd.get_name().to_string();

    for &shell in Shell::value_variants() {
        let path = generate_to(shell, &mut cmd, &name, &completions_dir)?;
        println!("cargo:information=completion completeion for {shell} generated in {path:?}");
    }

    Ok(())
}
