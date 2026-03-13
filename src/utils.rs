use regex::Regex;
use std::string::String;

pub fn dediac_ar(s: &str) -> String {
    let dediac_re =
        Regex::new(r"[\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}\u{0671}]")
            .unwrap();

    let sub = dediac_re.replace_all(s, "");
    sub.to_string()
}

pub fn bytes_to_mib_human_readable(num: usize) -> String {
    let num_mibs = num / (1024 * 1024);
    return num_mibs.to_string() + " MB";
}
