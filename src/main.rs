use anyhow::Context;
use anyhow::Result;
use clap::{Parser, Subcommand};
use fast_disambig::camel::downloader;
use fast_disambig::camel::mle;
use fast_disambig::camel::mle::disambiguate;
use fast_disambig::camel::morphology_db;
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
    let catalogue = downloader::CamelCatalogue::load()?;
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
            if let Some(word) = show {
                let resource = catalogue
                    .packages
                    .get("morphology-db-msa-r13")
                    .context("Could not find morphology_db-msa-r13 in catalogue")?;
                let db_path = resource.path.clone().context("Resource has no path")?;
                let db = morphology_db::MorphologyDB::load(db_path)?;
                let mle_resource = catalogue
                    .packages
                    .get("disambig-mle-calima-msa-r13")
                    .context("Could not find disambig-mle-calima-msa-r13 in catalogue")?;
                let mle_path = mle_resource
                    .path
                    .clone()
                    .context("Resource mle has no path")?;

                let test_file = std::fs::read_to_string(
                    "/Users/hadihamoud/Desktop/hadi/Work/DI/DRU/fast-disambig/test.txt",
                )?;

                let mle_model = mle::load_mle_model(mle_path)?;

                let tok_start = Instant::now();
                let test_words = utils::simple_word_tokenize(&test_file, "full");
                let tok_elapsed = tok_start.elapsed();

                let test_words_refs: Vec<&str> = test_words.iter().map(|s| s.as_str()).collect();

                let disambig_start = Instant::now();
                let result = disambiguate(&test_words_refs, &db, &mle_model, "NOAN_PROP", 1)?;
                let disambig_elapsed = disambig_start.elapsed();

                println!("Tokens: {}", test_words.len());
                println!("Tokenization: {:.3}ms", tok_elapsed.as_secs_f64() * 1000.0);
                println!(
                    "Disambiguation: {:.3}ms",
                    disambig_elapsed.as_secs_f64() * 1000.0
                );
                println!(
                    "Total: {:.3}ms",
                    (tok_elapsed + disambig_elapsed).as_secs_f64() * 1000.0
                );
            }
        }
    }
    Ok(())
}
