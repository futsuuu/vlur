use std::{
    env::{
        self,
        consts::{DLL_PREFIX, DLL_SUFFIX},
    },
    fs,
    io::BufReader,
    process::{Command, Stdio},
};

use clap::Parser as _;

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Subcommands,
    #[arg(global = true)]
    cargo_options: Vec<String>,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Wrapper of `cargo build`
    Build,
}

fn main() -> anyhow::Result<()> {
let args = Args::parse();
    println!("Start xtask...");

    match args.command {
        Subcommands::Build => {
            build(&args.cargo_options)?;
        }
    }

    println!("{}", "Finish xtask.");

    Ok(())
}

fn build(cargo_options: &[String]) -> anyhow::Result<()> {
    let mut command = Command::new("cargo")
        .arg("build")
        .args(["--message-format", "json-render-diagnostics"])
        .args(["--package", "vlur"])
        .args(cargo_options)
        .stdout(Stdio::piped())
        .spawn()?;

    let root_dir = env::current_dir()?;
    let out_dir = root_dir.join("bin");
    fs::create_dir_all(&out_dir)?;

    let reader = BufReader::new(command.stdout.take().unwrap());
    for message in cargo_metadata::Message::parse_stream(reader) {
        let cargo_metadata::Message::CompilerArtifact(artifact) = message.unwrap()
        else {
            continue;
        };
        if !artifact.target.src_path.starts_with(&root_dir) {
            continue;
        }

        let artifact_name = &artifact.target.name;
        let dll_name = &format!("{DLL_PREFIX}{artifact_name}{DLL_SUFFIX}");

        for generated in artifact.filenames {
            let file_name = generated.file_name().unwrap();

            if file_name != dll_name {
                continue;
            }

            fs::copy(
                generated,
                out_dir.join(format!(
                    "{artifact_name}{}",
                    match DLL_SUFFIX {
                        ".dylib" => ".so",
                        s => s,
                    }
                )),
            )?;
        }
    }

    Ok(())
}
