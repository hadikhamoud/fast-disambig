use clap::{Parser, Subcommand};
use fast_disambig::downloader;

#[derive(Parser)]
#[command(name = "fast-disambig", about = "CAMeL Tools CLI")]
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
        download: Option<String>,
    },
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let catalogue = downloader::CamelCatalogue::load()?;
    match cli.command {
        Commands::Data { list, download } => {
            if list {
                catalogue.display()?;
            }
            if let Some(package) = download {
                catalogue.download_resource(&package)?;
            }
        }
    }
    Ok(())
}
