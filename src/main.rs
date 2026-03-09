use fast_disambig::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = match utils::get_camel_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Could not get camel dir: {e}");
            return Err(e.into());
        }
    };

    let db_path = camel_dir.join("data/morphology_db/calima-msa-r13/morphology.db");
    let dediac = utils::dediac_ar(&db_path.to_string_lossy());
    println!("{dediac}");

    Ok(())
}
