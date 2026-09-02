use crate::camel::analyzer::ScoredAnalysis;
use crate::camel::mle;
use crate::camel::morphology_db::MorphologyDB;
use crate::utils;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

/// One output unit of `stem_tagged`: a stem piece, a separator, whitespace or punctuation.
///
/// `ud` is the Universal Dependencies tag of the piece (`SPEC_TOK` for separators,
/// `UNK` for whitespace or unanalysable words). `pos` is the CAMeL POS tag of the
/// word the piece came from. Concatenating `text` over all pieces reproduces `stem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub ud: String,
    pub pos: String,
}

/// Output target of the shared stemming loop. `stem` writes straight into a `String`,
/// `stem_tagged` collects `Piece`s. `WANTS_TAGS` is a compile-time constant, so the
/// tag alignment work is eliminated entirely from the `String` instantiation.
trait PieceSink {
    const WANTS_TAGS: bool;
    fn emit(&mut self, text: &str, ud: &str, pos: &str);
}

impl PieceSink for String {
    const WANTS_TAGS: bool = false;

    #[inline]
    fn emit(&mut self, text: &str, _ud: &str, _pos: &str) {
        self.push_str(text);
    }
}

impl PieceSink for Vec<Piece> {
    const WANTS_TAGS: bool = true;

    #[inline]
    fn emit(&mut self, text: &str, ud: &str, pos: &str) {
        self.push(Piece {
            text: text.to_owned(),
            ud: ud.to_owned(),
            pos: pos.to_owned(),
        });
    }
}

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

/// Emit segmented tokens with one UD tag per token, separators as `SPEC_TOK`.
/// UD tags are consumed in order and the last one is repeated once exhausted.
fn emit_tagged<S: PieceSink>(out: &mut S, toks: &[String], ud: &str, pos: &str, sep: &str) {
    let tags: Vec<&str> = ud.split('+').filter(|t| !t.is_empty()).collect();
    for (i, tok) in toks.iter().enumerate() {
        let tag = tags.get(i).or(tags.last()).copied().unwrap_or("UNK");
        let leading = tok.starts_with(sep);
        let core = tok.strip_prefix(sep).unwrap_or(tok);
        let trailing = core.ends_with(sep);
        let core = core.strip_suffix(sep).unwrap_or(core);
        if leading {
            out.emit(sep, "SPEC_TOK", pos);
        }
        if !core.is_empty() {
            out.emit(core, tag, pos);
        }
        if trailing {
            out.emit(sep, "SPEC_TOK", pos);
        }
    }
}

fn stem_into<S: PieceSink>(
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
    out: &mut S,
) -> Result<()> {
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

    let mut word_idx = 0;

    for token in &all_tokens {
        if !is_word_token(token) {
            let (ud, pos) = if S::WANTS_TAGS && !token.trim().is_empty() {
                ("PUNCT", "punc")
            } else {
                ("UNK", "UNK")
            };
            out.emit(token, ud, pos);
            continue;
        }

        let original = word_tokens[word_idx];
        let dediac = &dediac_words[word_idx];
        let word_analyses = cache.get(dediac.as_str()).unwrap();
        word_idx += 1;

        if word_analyses.is_empty() {
            out.emit(original, "UNK", "UNK");
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
                if S::WANTS_TAGS {
                    emit_tagged(out, &toks, &analysis.ud, &analysis.pos, sep);
                } else {
                    for t in &toks {
                        out.emit(t, "", "");
                    }
                }
            }
            None => {
                let ud = if S::WANTS_TAGS {
                    analysis
                        .ud
                        .split('+')
                        .find(|t| !t.is_empty())
                        .unwrap_or("X")
                } else {
                    ""
                };
                out.emit(original, ud, &analysis.pos);
            }
        }
    }

    Ok(())
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
    let mut output = String::new();
    stem_into(
        text,
        db,
        mle_model,
        sep,
        scheme,
        preserve_diacritics,
        backoff,
        cache,
        max_cache_size,
        fallback,
        &mut output,
    )?;
    Ok(output)
}

/// Same segmentation as `stem`, returned as tagged pieces instead of a joined string.
pub fn stem_tagged(
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
) -> Result<Vec<Piece>> {
    let mut output = Vec::new();
    stem_into(
        text,
        db,
        mle_model,
        sep,
        scheme,
        preserve_diacritics,
        backoff,
        cache,
        max_cache_size,
        fallback,
        &mut output,
    )?;
    Ok(output)
}
