use anyhow::Context;
use anyhow::Result;
use clap::{Parser, Subcommand};
use fast_disambig::camel::mle;
use fast_disambig::camel::mle::disambiguate;
use fast_disambig::camel::morphology_db;
use fast_disambig::sina::downloader;
use fast_disambig::utils;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fast-disambig", about = "Disambiguator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Data {
        #[arg(short, long)]
        list: bool,

        #[arg(short, long)]
        show: Option<String>,

        #[arg(short, long)]
        download: Option<String>,
    },
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let catalogue = downloader::SinaCatalogue::load()?;
    match cli.command {
        Commands::Data {
            list,
            download,
            show,
        } => {
            if list {
                catalogue.display()?;
            }
            if let Some(package) = download {
                catalogue.download_resource(&package)?;
            }
            if let Some(word) = show {}
        }
    }
    Ok(())
}
