use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::sync::LazyLock;
use zip::ZipArchive;

pub static JOIN_FEATS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["gloss", "bw"]));

pub static CONCAT_FEATS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["diac", "pattern", "caphi", "catib6", "ud"]));

pub static CONCAT_FEATS_NONE: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "d3tok", "d3seg", "atbseg", "d2seg", "d1seg", "d1tok", "d2tok", "atbtok", "bwtok",
    ])
});

pub static LOGPROB_FEATS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["pos_logprob", "lex_logprob", "pos_lex_logprob"]));

pub static TOK_SCHEMES_1: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "d1tok", "d2tok", "atbtok", "d1seg", "d2seg", "d3seg", "atbseg",
    ])
});

pub static TOK_SCHEMES_2: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["d3tok", "d3seg"]));

static RE_DEDIAC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}\u{0670}\u{0671}]",
    )
    .unwrap()
});
static RE_STRIP_LEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[_-]").unwrap());
pub static RE_ZERO_WIDTH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\u{200B}-\u{200D}\u{200E}\u{200F}\u{FEFF}]").unwrap());

static RE_ALEF_NORMALIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\u{0625}\u{0623}\u{0671}\u{0622}]").unwrap());
static RE_IS_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^.*[0-9\u{0660}-\u{0669}]+.*$").unwrap());
static RE_IS_STRICT_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9\u{0660}-\u{0669}]+$").unwrap());
static RE_IS_PUNC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[\p{P}\p{S}]+$").unwrap());
static RE_HAS_PUNC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{P}\p{S}]").unwrap());
static RE_IS_AR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^[\u{0621}\u{0622}\u{0623}\u{0624}\u{0625}\u{0626}\u{0627}",
        r"\u{0628}\u{0629}\u{062A}\u{062B}\u{062C}\u{062D}\u{062E}",
        r"\u{062F}\u{0630}\u{0631}\u{0632}\u{0633}\u{0634}\u{0635}",
        r"\u{0636}\u{0637}\u{0638}\u{0639}\u{063A}\u{0640}\u{0641}",
        r"\u{0642}\u{0643}\u{0644}\u{0645}\u{0646}\u{0647}\u{0648}",
        r"\u{0649}\u{064A}\u{0671}\u{067E}\u{0686}\u{06A4}\u{06AF}",
        r"\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}",
        r"\u{0670}]+$",
    ))
    .unwrap()
});

static RE_DIAC_1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"#\+*",
        r"([\u{062A}\u{062B}\u{062F}\u{0630}\u{0631}\u{0632}",
        r"\u{0633}\u{0634}\u{0635}\u{0636}\u{0637}\u{0638}\u{0644}",
        r"\u{0646}])",
    ))
    .unwrap()
});
static RE_DIAC_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#\+*").unwrap());
static RE_DIAC_3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\u{0627}\+?\u{064E}([\u{0629}\u{062A}])").unwrap());

static RE_CAPHI_1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(l-)\+",
        r"(t\_|th\_|d\_|th\.\_|r\_|z\_|",
        r"s\_|sh\_|s\.\_|d\.\_|t\.\_|",
        r"dh\.\_|l\_|n\_|dh\_)",
    ))
    .unwrap()
});
static RE_CAPHI_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\S)[-]*\+~").unwrap());
static RE_CAPHI_3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"i\_y-\+([^iau]+|$)").unwrap());
static RE_CAPHI_4: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"u\_w-\+([^iau]+|$)").unwrap());
static RE_CAPHI_5: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([iua])\+-2_[iua]").unwrap());
static RE_CAPHI_6: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(.+)\+-2_([iua])").unwrap());
static RE_CAPHI_7: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"u\+w(_+[^ioua])").unwrap());
static RE_CAPHI_8: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"p-\+([iua])").unwrap());
static RE_CAPHI_9: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"aa\+a[_]*").unwrap());

static RE_CAPHI_12: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"((^_+)|(_p?_*$))").unwrap());

static RE_TANWYN_FA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\u{064B}\u{0627}").unwrap());
static RE_TANWYN_FY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\u{064B}\u{0649}").unwrap());
static RE_TANWYN_AF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\u{0627}\u{064B}").unwrap());
static RE_TANWYN_YF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\u{0649}\u{064B}").unwrap());

static RE_TOKENIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{P}\p{S}]|[\p{L}\p{M}\p{N}]+|\s+").unwrap());
static RE_TOKENIZE_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{P}\p{S}]|[\p{N}]+|[\p{L}\p{M}]+|\s+").unwrap());
static RE_TOKENIZE_COMPACT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"[\p{P}\p{S}]",
        r"|[\p{N}]+",
        r"|[\u{0600}-\u{06FF}\u{0750}-\u{077F}\u{08A0}-\u{08FF}\u{FB50}-\u{FDFF}\u{FE70}-\u{FEFF}\p{M}]+",
        r"|[\p{L}]+",
        r"|\s+",
    ))
    .unwrap()
});

fn ar_to_caphi_char(c: char) -> Option<&'static str> {
    match c {
        '\u{0621}' => Some("2"),
        '\u{0622}' => Some("2_aa"),
        '\u{0623}' => Some("2"),
        '\u{0624}' => Some("2"),
        '\u{0625}' => Some("2"),
        '\u{0626}' => Some("2"),
        '\u{0627}' => Some("aa"),
        '\u{0628}' => Some("b"),
        '\u{062A}' => Some("t"),
        '\u{062B}' => Some("th"),
        '\u{062C}' => Some("j"),
        '\u{062D}' => Some("7"),
        '\u{062E}' => Some("kh"),
        '\u{062F}' => Some("d"),
        '\u{0630}' => Some("dh"),
        '\u{0631}' => Some("r"),
        '\u{0632}' => Some("z"),
        '\u{0633}' => Some("s"),
        '\u{0634}' => Some("sh"),
        '\u{0635}' => Some("s."),
        '\u{0636}' => Some("d."),
        '\u{0637}' => Some("t."),
        '\u{0638}' => Some("dh."),
        '\u{0639}' => Some("3"),
        '\u{063A}' => Some("gh"),
        '\u{0641}' => Some("f"),
        '\u{0642}' => Some("q"),
        '\u{0643}' => Some("k"),
        '\u{0644}' => Some("l"),
        '\u{0645}' => Some("m"),
        '\u{0646}' => Some("n"),
        '\u{0647}' => Some("h"),
        '\u{0648}' => Some("w"),
        '\u{0649}' => Some("aa"),
        '\u{064A}' => Some("y"),
        _ => None,
    }
}

pub fn dediac_ar(s: &str) -> Result<String> {
    Ok(RE_DEDIAC.replace_all(s, "").to_string())
}

pub fn strip_lex(s: &str) -> Result<String> {
    Ok(RE_STRIP_LEX.split(s).next().unwrap_or(s).to_string())
}

pub fn normalize_alef_ar(s: &str) -> String {
    RE_ALEF_NORMALIZE.replace_all(s, "\u{0627}").to_string()
}

pub fn normalize_alef_maksura_ar(s: &str) -> String {
    s.replace('\u{0649}', "\u{064A}")
}

pub fn normalize_teh_marbuta_ar(s: &str) -> String {
    s.replace('\u{0629}', "\u{0647}")
}

pub fn normalize_ar(s: &str) -> Result<String> {
    let s = dediac_ar(s)?;
    let s = normalize_alef_ar(&s);
    let s = normalize_alef_maksura_ar(&s);
    let s = normalize_teh_marbuta_ar(&s);
    Ok(s)
}

pub fn is_digit(word: &str) -> bool {
    RE_IS_DIGIT.is_match(word)
}

pub fn is_strict_digit(word: &str) -> bool {
    RE_IS_STRICT_DIGIT.is_match(word)
}

pub fn is_punc(word: &str) -> bool {
    RE_IS_PUNC.is_match(word)
}

pub fn has_punc(word: &str) -> bool {
    RE_HAS_PUNC.is_match(word)
}

pub fn is_ar(word: &str) -> bool {
    RE_IS_AR.is_match(word)
}

pub fn simple_ar_to_caphi(ar_str: &str) -> String {
    let mut chars: Vec<char> = ar_str.chars().collect();
    if chars.first() == Some(&'\u{0627}') {
        chars[0] = '\u{0623}';
    }
    chars
        .iter()
        .filter_map(|c| ar_to_caphi_char(*c))
        .collect::<Vec<&str>>()
        .join("_")
}

pub fn normalize_tanwyn(word: &str, mode: &str) -> String {
    let mut result = word.to_string();
    if mode == "FA" {
        result = RE_TANWYN_FA
            .replace_all(&result, "\u{064B}\u{0627}")
            .to_string();
        result = RE_TANWYN_FY
            .replace_all(&result, "\u{064B}\u{0649}")
            .to_string();
    } else {
        result = RE_TANWYN_AF
            .replace_all(&result, "\u{0627}\u{064B}")
            .to_string();
        result = RE_TANWYN_YF
            .replace_all(&result, "\u{0649}\u{064B}")
            .to_string();
    }
    result
}

fn dedup_char(s: &str, c: char) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_c = false;
    for ch in s.chars() {
        if ch == c {
            if !prev_was_c {
                result.push(ch);
            }
            prev_was_c = true;
        } else {
            prev_was_c = false;
            result.push(ch);
        }
    }
    result
}

pub fn rewrite_diac(word: &str) -> String {
    let mut result = RE_DIAC_1.replace_all(word, "${1}\u{0651}").to_string();
    result = RE_DIAC_2.replace_all(&result, "").to_string();
    result = RE_DIAC_3.replace_all(&result, "\u{0627}${1}").to_string();
    result = result.replace('\u{0671}', "\u{0627}");
    result = result.replace('+', "");
    result = dedup_char(&result, '\u{0651}');
    result
}

pub fn rewrite_caphi(word: &str) -> String {
    let mut result = RE_CAPHI_1.replace_all(word, "${2}${2}").to_string();
    result = RE_CAPHI_2.replace_all(&result, "${1}_${1}").to_string();
    result = RE_CAPHI_3.replace_all(&result, "ii_${1}").to_string();
    result = RE_CAPHI_4.replace_all(&result, "uu_${1}").to_string();
    result = RE_CAPHI_5.replace_all(&result, "${1}").to_string();
    result = RE_CAPHI_6.replace_all(&result, "${1}_${2}").to_string();
    result = RE_CAPHI_7.replace_all(&result, "uu${1}").to_string();
    result = RE_CAPHI_8.replace_all(&result, "t_${1}").to_string();
    result = RE_CAPHI_9.replace_all(&result, "aa_").to_string();
    result = result.replace('+', "_").replace('-', "_");
    result = dedup_char(&result, '_');
    result = RE_CAPHI_12.replace_all(&result, "").to_string();
    result
}

pub fn rewrite_tok_1(word: &str) -> String {
    let mut result = RE_DIAC_1.replace_all(word, "${1}\u{0651}").to_string();
    result = RE_DIAC_2.replace_all(&result, "").to_string();
    result = RE_DIAC_3.replace_all(&result, "\u{0627}${1}").to_string();
    result
}

pub fn rewrite_tok_2(word: &str) -> String {
    RE_DIAC_3.replace_all(word, "\u{0627}${1}").to_string()
}

pub fn rewrite_pattern(word: &str) -> String {
    RE_DIAC_2.replace_all(word, "").to_string()
}

pub fn replace_noan(s: &str, replacement: &str) -> String {
    s.replace("NOAN", replacement)
}

pub fn join_non_empty(parts: &[&str], sep: &str) -> String {
    parts
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<&str>>()
        .join(sep)
}

pub fn pick_override<'a>(base: &'a str, candidate: &'a str) -> &'a str {
    if candidate != "-" && !candidate.is_empty() {
        candidate
    } else {
        base
    }
}

pub fn apply_override(field: &mut String, candidate: &str) {
    if candidate != "-" && !candidate.is_empty() {
        field.clear();
        field.push_str(candidate);
    }
}

pub fn simple_word_tokenize(sentence: &str, mode: &str) -> Vec<String> {
    let re = match mode {
        "full" => &*RE_TOKENIZE,
        "full_split_digits" => &*RE_TOKENIZE_NUMBER,
        _ => &*RE_TOKENIZE_COMPACT,
    };
    re.find_iter(sentence)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn bytes_to_mib_human_readable(num: usize) -> String {
    let num_mibs = num / (1024 * 1024);
    return num_mibs.to_string() + " MB";
}

pub fn unzip_file(zip_path: &PathBuf, extract_to_path: &PathBuf) -> Result<()> {
    let file = fs::File::open(zip_path)
        .context(format!("Failed to open zip file {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;
    archive.extract(extract_to_path).context(format!(
        "Failed to extract to {}",
        extract_to_path.display()
    ))?;
    Ok(())
}

pub fn hash(inp: &[u8]) -> String {
    let digest = Sha256::digest(inp);
    format!("{:x}", digest)
}

pub const TAA_MARBOUTA: char = '\u{0629}';
pub const TAA_MARBOUTA_DETACHED: char = '\u{FE93}';
pub const HAA: char = '\u{0647}';

pub static DIACRITIC_SET: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        '\u{064B}', '\u{064C}', '\u{064D}', '\u{064E}', '\u{064F}', '\u{0650}', '\u{0651}',
        '\u{0652}', '\u{0670}',
    ])
});

pub fn has_diacritics(word: &str) -> bool {
    word.chars().any(|c| DIACRITIC_SET.contains(&c))
}

pub fn is_diacritic(c: char) -> bool {
    DIACRITIC_SET.contains(&c)
}

pub fn split_and_replace_sep(tok: &str, sep: &str) -> Vec<String> {
    let parts: Vec<&str> = tok.split('_').collect();
    let mut result = Vec::new();
    for part in parts.iter() {
        let mut s = part.to_string();
        if s.starts_with('+') {
            s = format!("{}{}", sep, &s[1..]);
        }
        if s.ends_with('+') {
            s = format!("{}{}", &s[..s.len() - 1], sep);
        }
        result.push(s);
    }
    result
}

pub fn split_token_on_t(toks: Vec<String>, sep: &str) -> Vec<String> {
    let mut result = Vec::new();
    for tok in toks {
        let last = tok.chars().last().unwrap_or('\0');
        if last == TAA_MARBOUTA || last == TAA_MARBOUTA_DETACHED || last == HAA {
            if tok.chars().count() == 1 && last == HAA {
                result.push(format!("{}{}", sep, TAA_MARBOUTA));
            } else {
                let without_last: String = tok.chars().take(tok.chars().count() - 1).collect();
                result.push(without_last);
                result.push(format!("{}{}", sep, last));
            }
        } else {
            result.push(tok);
        }
    }
    result
}

pub fn merge_alef_lam(toks: Vec<String>, sep: &str) -> Vec<String> {
    let lam_plus = format!("ل{}", sep);
    let alef_lam_plus = format!("ال{}", sep);
    let lam_lam_plus = format!("لل{}", sep);

    let mut result = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if i + 1 < toks.len() && toks[i] == lam_plus && toks[i + 1] == alef_lam_plus {
            result.push(lam_lam_plus.clone());
            i += 2;
            continue;
        }
        result.push(toks[i].clone());
        i += 1;
    }
    result
}

pub fn merge_tokens(toks: &[String], sep: &str) -> String {
    let sep_len = sep.len();
    let mut parts = Vec::new();
    for tok in toks {
        if tok == sep {
            parts.push("_".to_string());
        } else if tok.ends_with(sep) {
            parts.push(tok[..tok.len() - sep_len].to_string());
        } else if tok.starts_with(sep) {
            parts.push(tok[sep_len..].to_string());
        } else {
            parts.push(tok.clone());
        }
    }
    parts.join("")
}

pub fn apply_diacritics(segments: &[String], diacritized: &str, sep: &str) -> Vec<String> {
    let mut source = diacritized.chars().peekable();
    let mut result = Vec::new();

    let leading: String = diacritized
        .chars()
        .take_while(|c| is_diacritic(*c))
        .collect();
    for _ in 0..leading.chars().count() {
        source.next();
    }

    for (seg_idx, seg) in segments.iter().enumerate() {
        if seg == sep {
            result.push(sep.to_string());
            continue;
        }

        let mut out = String::new();
        if seg_idx == 0 && !leading.is_empty() {
            out.push_str(&leading);
        }

        let sep_bytes = sep.len();
        let mut pos = 0;

        while pos < seg.len() {
            if pos + sep_bytes <= seg.len()
                && seg.is_char_boundary(pos)
                && seg.is_char_boundary(pos + sep_bytes)
                && &seg[pos..pos + sep_bytes] == sep
            {
                out.push_str(sep);
                pos += sep_bytes;
                continue;
            }

            let c = seg[pos..].chars().next().unwrap();

            while source.peek().map_or(false, |sc| is_diacritic(*sc)) {
                out.push(source.next().unwrap());
            }

            if source.peek() == Some(&c) {
                out.push(source.next().unwrap());
                while source.peek().map_or(false, |sc| is_diacritic(*sc)) {
                    out.push(source.next().unwrap());
                }
            } else {
                out.push(c);
            }

            pos += c.len_utf8();
        }

        result.push(out);
    }
    result
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];

    for i in 0..=a.len() {
        dp[i][0] = i;
    }

    for j in 0..=b.len() {
        dp[0][j] = j;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };

            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    dp[a.len()][b.len()]
}
