use crate::camel::analyzer::ScoredAnalysis;
use crate::camel::mle;
use crate::camel::morphology_db::MorphologyDB;
use crate::utils;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

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

pub fn cache_keys(text: &str) -> Vec<String> {
    let text = text.replace('\u{0640}', "");
    let text = utils::RE_ZERO_WIDTH.replace_all(&text, "");
    let mut seen = HashSet::new();

    utils::simple_word_tokenize(&text, "full")
        .into_iter()
        .filter(|token| is_word_token(token))
        .filter_map(|word| utils::dediac_ar(&word).ok())
        .filter(|word| seen.insert(word.clone()))
        .collect()
}

fn try_scheme(
    analysis: &ScoredAnalysis,
    scheme: &str,
    dediac: &str,
    sep: &str,
    ends_with_ta: bool,
) -> Option<Vec<String>> {
    let tok_raw = get_scheme_field(analysis, scheme);
    if tok_raw.is_empty() || tok_raw.contains("NOAN") {
        return None;
    }

    let tok = utils::dediac_ar(tok_raw).ok()?;
    let mut toks = utils::split_and_replace_sep(&tok, sep);

    if ends_with_ta {
        toks = utils::split_token_on_t(toks, sep);
    }

    toks = utils::merge_alef_lam(toks, sep);
    let merged = utils::merge_tokens(&toks, sep);

    if merged == dediac && toks.len() > 1 {
        Some(toks)
    } else {
        None
    }
}

pub fn stem(
    text: &str,
    db: &MorphologyDB,
    mle_model: &HashMap<String, ScoredAnalysis>,
    sep: &str,
    scheme: &str,
    preserve_diacritics: bool,
    backoff: &str,
    cache: Option<&mut HashMap<String, Vec<ScoredAnalysis>>>,
    max_cache_size: usize,
    fallback: &[&str],
) -> Result<String> {
    let text = text.replace('\u{0640}', "");
    let text = utils::RE_ZERO_WIDTH.replace_all(&text, "").to_string();

    let all_tokens = utils::simple_word_tokenize(&text, "full");

    let word_tokens: Vec<&str> = all_tokens
        .iter()
        .filter(|t| is_word_token(t))
        .map(|s| s.as_str())
        .collect();

    let dediac_words: Vec<String> = word_tokens
        .iter()
        .map(|w| utils::dediac_ar(w).unwrap_or_default())
        .collect();

    let mut local_cache: HashMap<String, Vec<ScoredAnalysis>> = HashMap::new();
    let cache = match cache {
        Some(c) => {
            if max_cache_size > 0 && c.len() > max_cache_size {
                c.clear();
            }
            c
        }
        None => &mut local_cache,
    };

    let mut uncached: Vec<&str> = Vec::new();
    let mut seen = HashSet::new();
    for dediac in &dediac_words {
        if !cache.contains_key(dediac.as_str()) && seen.insert(dediac.as_str()) {
            uncached.push(dediac.as_str());
        }
    }

    if !uncached.is_empty() {
        let new_results: Vec<(&str, Vec<ScoredAnalysis>)> = uncached
            .par_iter()
            .map(|w| {
                let result =
                    mle::disambiguate_word(w, db, mle_model, backoff, 1).unwrap_or_default();
                (*w, result)
            })
            .collect();

        for (word, result) in new_results {
            cache.insert(word.to_string(), result);
        }
    }

    let mut output = String::new();
    let mut word_idx = 0;

    for token in &all_tokens {
        if !is_word_token(token) {
            output.push_str(token);
            continue;
        }

        let original = word_tokens[word_idx];
        let dediac = &dediac_words[word_idx];
        let word_analyses = cache.get(dediac.as_str()).unwrap();
        word_idx += 1;

        if word_analyses.is_empty() {
            output.push_str(original);
            continue;
        }

        let analysis = &word_analyses[0];
        let word_has_diacritics = preserve_diacritics && utils::has_diacritics(original);
        let ends_with_ta =
            dediac.ends_with(utils::TAA_MARBOUTA) || dediac.ends_with(utils::TAA_MARBOUTA_DETACHED);

        let mut resolved = try_scheme(analysis, scheme, dediac, sep, ends_with_ta);

        if resolved.is_none() && !fallback.is_empty() {
            for fb in fallback {
                resolved = try_scheme(analysis, fb, dediac, sep, ends_with_ta);
                if resolved.is_some() {
                    break;
                }
            }
        }

        match resolved {
            Some(mut toks) => {
                if word_has_diacritics {
                    toks = utils::apply_diacritics(&toks, original, sep);
                }
                for t in &toks {
                    output.push_str(t);
                }
            }
            None => {
                output.push_str(original);
            }
        }
    }

    Ok(output)
}
