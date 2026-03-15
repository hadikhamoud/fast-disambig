use crate::analyzer::ScoredAnalysis;
use crate::mle;
use crate::morphology_db::MorphologyDB;
use crate::utils;
use anyhow::Result;
use std::collections::HashMap;

fn get_scheme_field<'a>(analysis: &'a ScoredAnalysis, scheme: &str) -> &'a str {
    match scheme {
        "d1seg" => &analysis.d1seg,
        "d2seg" => &analysis.d2seg,
        "d3seg" => &analysis.d3seg,
        "atbseg" => &analysis.atbseg,
        "d1tok" => &analysis.d1tok,
        "d2tok" => &analysis.d2tok,
        "d3tok" => &analysis.d3tok,
        "atbtok" => &analysis.atbtok,
        "bwtok" => &analysis.bwtok,
        _ => &analysis.d3tok,
    }
}

fn is_word_token(s: &str) -> bool {
    s.chars().next().map_or(false, |c| {
        c.is_alphanumeric() || ('\u{0600}'..='\u{FEFF}').contains(&c)
    })
}

pub fn stem(
    text: &str,
    db: &MorphologyDB,
    mle_model: &HashMap<String, ScoredAnalysis>,
    sep: &str,
    scheme: &str,
    preserve_diacritics: bool,
    backoff: &str,
) -> Result<String> {
    let text = text.replace('\u{0640}', "");
    let text = utils::RE_ZERO_WIDTH.replace_all(&text, "").to_string();

    let all_tokens = utils::simple_word_tokenize(&text, "compact");

    let word_tokens: Vec<&str> = all_tokens
        .iter()
        .filter(|t| is_word_token(t))
        .map(|s| s.as_str())
        .collect();

    let disambig_results = mle::disambiguate(&word_tokens, db, mle_model, backoff, 1)?;

    let mut output = String::new();
    let mut word_idx = 0;

    for token in &all_tokens {
        if !is_word_token(token) {
            output.push_str(token);
            continue;
        }

        let original = word_tokens[word_idx];
        let word_analyses = &disambig_results[word_idx];
        word_idx += 1;

        if word_analyses.is_empty() {
            output.push_str(original);
            continue;
        }

        let analysis = &word_analyses[0];
        let dediac_word = utils::dediac_ar(original)?;
        let word_has_diacritics = preserve_diacritics && utils::has_diacritics(original);

        let tok_raw = get_scheme_field(analysis, scheme);
        if tok_raw.is_empty() || tok_raw.contains("NOAN") {
            output.push_str(original);
            continue;
        }

        let tok = utils::dediac_ar(tok_raw)?;
        let ends_with_ta = dediac_word.ends_with(utils::TAA_MARBOUTA)
            || dediac_word.ends_with(utils::TAA_MARBOUTA_DETACHED);

        let mut toks = utils::split_and_replace_sep(&tok, sep);

        if ends_with_ta {
            toks = utils::split_token_on_t(toks, sep);
        }

        toks = utils::merge_alef_lam(toks, sep);
        let merged = utils::merge_tokens(&toks, sep);

        if merged == dediac_word && toks.len() > 1 {
            if word_has_diacritics {
                toks = utils::apply_diacritics(&toks, original, sep);
            }
            for t in &toks {
                output.push_str(t);
            }
        } else {
            output.push_str(original);
        }
    }

    Ok(output)
}
