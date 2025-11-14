use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use solify::commands::{gen_test, inspect};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ABOUT: &str = "Solify - A CLI tool to generate anchor program tests";

#[derive(Parser)]
#[command(name = "solify")]
#[command(version = VERSION)]
#[command(about = ABOUT, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,
}

#[derive(Subcommand)]
enum Commands {
    Inspect {
        signature: String,
    },
    GenTest {
        #[arg(short, long, default_value = "./target/idl")]
        idl: PathBuf,
        #[arg(short = 'o', long, default_value = "./tests")]
        output: PathBuf,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let cli = Cli::parse();
    if cli.verbose {
        log::info!("Verbose mode enabled");
    }

    match cli.command {
        Commands::Inspect {
            signature,
        } => {
            inspect::execute(signature, &cli.rpc_url).await?;
        }
        Commands::GenTest { idl, output } => {
            gen_test::execute(idl,output, &cli.rpc_url).await?;
        }
    }
    Ok(())
}

