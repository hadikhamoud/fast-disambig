use regex::Regex;
use std::string::String;

pub fn dediac_ar(s: &str) -> String {
    let dediac_re =
        Regex::new(r"\u064b|\u064c|\u064d|\u064e|\u064f|\u0650|\u0651|\u0652u|\u0671").unwrap();

    let sub = dediac_re.replace_all(s, "");
    sub.to_string()
}
