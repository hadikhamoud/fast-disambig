use anyhow::Result;
use rayon::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::camel::analyzer::{self, ScoredAnalysis};
use crate::camel::morphology_db::MorphologyDB;
use crate::utils;

pub fn load_mle_model(model_path: PathBuf) -> Result<HashMap<String, ScoredAnalysis>> {
    let model_path = if model_path.is_dir() {
        model_path.join("model.json")
    } else {
        model_path
    };
    let json_str = std::fs::read_to_string(model_path)?;
    let mut mle_model: HashMap<String, ScoredAnalysis> = serde_json::from_str(&json_str)?;
    for analysis in mle_model.values_mut() {
        analysis.lex = utils::strip_lex(&analysis.lex)?;
    }
    Ok(mle_model)
}

fn distance_score(a: &str, r: &str) -> f64 {
    if r.is_empty() {
        return 0.0;
    }
    let distance = utils::levenshtein(r, a);
    let len = r.chars().count();
    (len as f64 - distance as f64).max(0.0) / len as f64
}

pub fn score_analysis(analysis: &ScoredAnalysis, reference: &ScoredAnalysis) -> f64 {
    let mut score: f64 = 0.0;

    score += (analysis.asp == reference.asp) as u8 as f64;
    score += (analysis.cas == reference.cas) as u8 as f64;
    score += (analysis.enc0 == reference.enc0) as u8 as f64;
    score += (analysis.enc1 == reference.enc1) as u8 as f64;
    score += (analysis.enc2 == reference.enc2) as u8 as f64;
    score += (analysis.r#gen == reference.r#gen) as u8 as f64;
    score += (analysis.r#mod == reference.r#mod) as u8 as f64;
    score += (analysis.num == reference.num) as u8 as f64;
    score += (analysis.per == reference.per) as u8 as f64;
    score += (analysis.pos == reference.pos) as u8 as f64;
    score += (analysis.prc0 == reference.prc0) as u8 as f64;
    score += (analysis.prc1 == reference.prc1) as u8 as f64;
    score += (analysis.prc2 == reference.prc2) as u8 as f64;
    score += (analysis.prc3 == reference.prc3) as u8 as f64;
    score += (analysis.form_num == reference.form_num) as u8 as f64;
    score += (analysis.form_gen == reference.form_gen) as u8 as f64;
    score += (analysis.stt == reference.stt) as u8 as f64;
    score += (analysis.vox == reference.vox) as u8 as f64;

    score += distance_score(&analysis.diac, &reference.diac);
    score += distance_score(&analysis.lex, &reference.lex);
    score += distance_score(&analysis.bw, &reference.bw);

    score
}

pub fn disambiguate_word(
    word: &str,
    db: &MorphologyDB,
    mle_model: &HashMap<String, ScoredAnalysis>,
    backoff: &str,
    top: usize,
) -> Result<Vec<ScoredAnalysis>> {
    let word = utils::dediac_ar_cow(word);

    let mut analyses: Vec<analyzer::AnalysisCandidate<'_>> =
        analyzer::analyze_for_disambiguation(&word, db, false, backoff)?
            .into_iter()
            .collect();

    if analyses.is_empty() {
        return Ok(vec![]);
    }

    if let Some(mle_analysis) = mle_model.get(word.as_ref()) {
        for a in &mut analyses {
            a.analysis.score = score_analysis(&a.analysis, mle_analysis);
        }

        analyses.sort_by(|a, b| {
            b.analysis
                .score
                .partial_cmp(&a.analysis.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.analysis.bw.len().cmp(&b.analysis.bw.len()))
                .then_with(|| a.analysis.diac.cmp(&b.analysis.diac))
        });

        let max_score = analyses.first().map(|a| a.analysis.score).unwrap_or(1.0);
        let max_score = if max_score == 0.0 { 1.0 } else { max_score };

        for a in &mut analyses {
            a.analysis.score /= max_score;
        }
    } else {
        let max_prob = analyses
            .iter()
            .map(|a| 10_f64.powf(a.analysis.pos_lex_logprob))
            .fold(f64::NEG_INFINITY, f64::max);

        let max_prob = if max_prob == 0.0 { 1.0 } else { max_prob };

        for a in &mut analyses {
            a.analysis.score = 10_f64.powf(a.analysis.pos_lex_logprob) / max_prob;
        }

        analyses.sort_by(|a, b| {
            b.analysis
                .score
                .partial_cmp(&a.analysis.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.analysis
                        .pos_lex_logprob
                        .partial_cmp(&a.analysis.pos_lex_logprob)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    b.analysis
                        .lex_logprob
                        .partial_cmp(&a.analysis.lex_logprob)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.analysis.diac.cmp(&b.analysis.diac))
        });
    }

    analyses.truncate(top);
    analyses
        .into_iter()
        .map(|candidate| candidate.materialize(db))
        .collect()
}

pub fn disambiguate(
    sentence: &[&str],
    db: &MorphologyDB,
    mle_model: &HashMap<String, ScoredAnalysis>,
    backoff: &str,
    top: usize,
) -> Result<Vec<Vec<ScoredAnalysis>>> {
    sentence
        .par_iter()
        .map(|w| disambiguate_word(w, db, mle_model, backoff, top))
        .collect()
}
