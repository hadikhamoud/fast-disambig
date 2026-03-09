use regex::Regex;
use std::{env, fs, io, path::PathBuf, string::String};

pub fn get_camel_dir() -> io::Result<PathBuf> {
    let home_dir = env::var("HOME").map(PathBuf::from).map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME environment variable is not set",
        )
    })?;

    let camel_dir = home_dir.join(".camel_tools");
    if !camel_dir.exists() {
        fs::create_dir_all(&camel_dir)?;
    }

    Ok(camel_dir)
}

pub fn dediac_ar(s: &str) -> String {
    let dediac_re =
        Regex::new(r"[\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}\u{0671}]")
            .unwrap();

    let sub = dediac_re.replace_all(s, "");
    sub.to_string()
}
