use fast_disambig::downloader;
use fast_disambig::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = match downloader::get_or_create_camel_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Could not get camel dir: {e}");
            return Err(e.into());
        }
    };

    let db_path = camel_dir.join("data/morphology_db/calima-msa-r13/morphology.db");
    let dediac = utils::dediac_ar(&db_path.to_string_lossy());
    downloader::get_camel_catalogue()?;
    println!("{dediac}");

    Ok(())
}
