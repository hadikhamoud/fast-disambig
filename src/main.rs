use fast_disambig::downloader;
use fast_disambig::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = downloader::get_or_create_camel_dir()?;

    let db_path = camel_dir.join("data/morphology_db/calima-msa-r13/morphology.db");
    let dediac = utils::dediac_ar(&db_path.to_string_lossy());
    let catalogue = downloader::CamelCatalogue::load()?;
    catalogue.display()?;

    Ok(())
}
