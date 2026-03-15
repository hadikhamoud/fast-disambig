use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::morphology_db;
use crate::utils;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct ScoredAnalysis {
    #[serde(default)]
    pub score: f64,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub diac: String,
    #[serde(default)]
    pub lex: String,
    #[serde(default)]
    pub bw: String,
    #[serde(default)]
    pub gloss: String,
    #[serde(default)]
    pub stem: String,
    #[serde(default)]
    pub stemcat: String,
    #[serde(default)]
    pub stemgloss: String,
    #[serde(default)]
    pub catib6: String,
    #[serde(default)]
    pub ud: String,
    #[serde(default)]
    pub per: String,
    #[serde(default)]
    pub asp: String,
    #[serde(default)]
    pub vox: String,
    #[serde(default, rename = "mod")]
    pub r#mod: String,
    #[serde(default, rename = "gen")]
    pub r#gen: String,
    #[serde(default)]
    pub num: String,
    #[serde(default)]
    pub stt: String,
    #[serde(default)]
    pub cas: String,
    #[serde(default)]
    pub rat: String,
    #[serde(default)]
    pub form_gen: String,
    #[serde(default)]
    pub form_num: String,
    #[serde(default)]
    pub pos: String,
    #[serde(default)]
    pub prc3: String,
    #[serde(default)]
    pub prc2: String,
    #[serde(default)]
    pub prc1: String,
    #[serde(default)]
    pub prc0: String,
    #[serde(default)]
    pub enc0: String,
    #[serde(default)]
    pub d1seg: String,
    #[serde(default)]
    pub d2seg: String,
    #[serde(default)]
    pub d3seg: String,
    #[serde(default)]
    pub atbseg: String,
    #[serde(default)]
    pub d1tok: String,
    #[serde(default)]
    pub d2tok: String,
    #[serde(default)]
    pub d3tok: String,
    #[serde(default)]
    pub atbtok: String,
    #[serde(default)]
    pub bwtok: String,
    #[serde(default)]
    pub root: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub caphi: String,
    #[serde(default)]
    pub pos_logprob: f64,
    #[serde(default)]
    pub lex_logprob: f64,
    #[serde(default)]
    pub pos_lex_logprob: f64,
}

impl ScoredAnalysis {
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self> {
        let logprob_keys = ["pos_logprob", "lex_logprob", "pos_lex_logprob"];
        let deserializer = serde::de::value::MapDeserializer::<_, serde::de::value::Error>::new(
            map.iter()
                .filter(|(k, v)| v.as_str() != "*" && !logprob_keys.contains(&k.as_str()))
                .map(|(k, v)| (k.as_str(), v.as_str())),
        );
        let mut sa = ScoredAnalysis::deserialize(deserializer)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ScoredAnalysis: {}", e))?;

        if let Some(v) = map.get("pos_logprob") {
            sa.pos_logprob = v.parse().unwrap_or(0.0);
        }
        if let Some(v) = map.get("lex_logprob") {
            sa.lex_logprob = v.parse().unwrap_or(0.0);
        }
        if let Some(v) = map.get("pos_lex_logprob") {
            sa.pos_lex_logprob = v.parse().unwrap_or(0.0);
        }

        Ok(sa)
    }

    pub fn set_tok_all(&mut self, val: &str) {
        self.d1seg = val.to_string();
        self.d2seg = val.to_string();
        self.d3seg = val.to_string();
        self.atbseg = val.to_string();
        self.d1tok = val.to_string();
        self.d2tok = val.to_string();
        self.d3tok = val.to_string();
        self.atbtok = val.to_string();
        self.bwtok = val.to_string();
    }

    pub fn set_lex_feats_all(&mut self, val: &str) {
        self.root = val.to_string();
        self.pattern = val.to_string();
        self.caphi = val.to_string();
    }

    pub fn set_logprobs_all(&mut self, val: f64) {
        self.pos_logprob = val;
        self.lex_logprob = val;
        self.pos_lex_logprob = val;
    }
}

pub fn merge_features(
    db: &morphology_db::MorphologyDB,
    prefix: &ScoredAnalysis,
    stem: &ScoredAnalysis,
    suffix: &ScoredAnalysis,
    join_char: &str,
    diac_mode: &str,
) -> Result<ScoredAnalysis> {
    let mut result = stem.clone();

    result.pos = utils::pick_override(&result.pos, &suffix.pos).to_string();
    result.pos = utils::pick_override(&result.pos, &prefix.pos).to_string();

    result.per = utils::pick_override(&result.per, &suffix.per).to_string();
    result.per = utils::pick_override(&result.per, &prefix.per).to_string();

    result.asp = utils::pick_override(&result.asp, &suffix.asp).to_string();
    result.asp = utils::pick_override(&result.asp, &prefix.asp).to_string();

    result.vox = utils::pick_override(&result.vox, &suffix.vox).to_string();
    result.vox = utils::pick_override(&result.vox, &prefix.vox).to_string();

    result.r#mod = utils::pick_override(&result.r#mod, &suffix.r#mod).to_string();
    result.r#mod = utils::pick_override(&result.r#mod, &prefix.r#mod).to_string();

    result.r#gen = utils::pick_override(&result.r#gen, &suffix.r#gen).to_string();
    result.r#gen = utils::pick_override(&result.r#gen, &prefix.r#gen).to_string();

    result.num = utils::pick_override(&result.num, &suffix.num).to_string();
    result.num = utils::pick_override(&result.num, &prefix.num).to_string();

    result.stt = utils::pick_override(&result.stt, &suffix.stt).to_string();
    result.stt = utils::pick_override(&result.stt, &prefix.stt).to_string();

    result.cas = utils::pick_override(&result.cas, &suffix.cas).to_string();
    result.cas = utils::pick_override(&result.cas, &prefix.cas).to_string();

    result.rat = utils::pick_override(&result.rat, &suffix.rat).to_string();
    result.rat = utils::pick_override(&result.rat, &prefix.rat).to_string();

    result.form_gen = utils::pick_override(&result.form_gen, &suffix.form_gen).to_string();
    result.form_gen = utils::pick_override(&result.form_gen, &prefix.form_gen).to_string();

    result.form_num = utils::pick_override(&result.form_num, &suffix.form_num).to_string();
    result.form_num = utils::pick_override(&result.form_num, &prefix.form_num).to_string();

    result.prc3 = utils::pick_override(&result.prc3, &suffix.prc3).to_string();
    result.prc3 = utils::pick_override(&result.prc3, &prefix.prc3).to_string();

    result.prc2 = utils::pick_override(&result.prc2, &suffix.prc2).to_string();
    result.prc2 = utils::pick_override(&result.prc2, &prefix.prc2).to_string();

    result.prc1 = utils::pick_override(&result.prc1, &suffix.prc1).to_string();
    result.prc1 = utils::pick_override(&result.prc1, &prefix.prc1).to_string();

    result.prc0 = utils::pick_override(&result.prc0, &suffix.prc0).to_string();
    result.prc0 = utils::pick_override(&result.prc0, &prefix.prc0).to_string();

    result.enc0 = utils::pick_override(&result.enc0, &suffix.enc0).to_string();
    result.enc0 = utils::pick_override(&result.enc0, &prefix.enc0).to_string();

    result.source = utils::pick_override(&result.source, &suffix.source).to_string();
    result.source = utils::pick_override(&result.source, &prefix.source).to_string();

    result.lex = utils::pick_override(&result.lex, &suffix.lex).to_string();
    result.lex = utils::pick_override(&result.lex, &prefix.lex).to_string();

    result.root = utils::pick_override(&result.root, &suffix.root).to_string();
    result.root = utils::pick_override(&result.root, &prefix.root).to_string();

    if db.defines.contains_key("gloss") {
        result.gloss =
            utils::join_non_empty(&[&prefix.gloss, &stem.gloss, &suffix.gloss], join_char);
    }
    if db.defines.contains_key("bw") {
        result.bw = utils::join_non_empty(&[&prefix.bw, &stem.bw, &suffix.bw], join_char);
    }

    if db.defines.contains_key("diac") {
        result.diac = utils::join_non_empty(&[&prefix.diac, &stem.diac, &suffix.diac], join_char);
    }
    if db.defines.contains_key("catib6") {
        result.catib6 =
            utils::join_non_empty(&[&prefix.catib6, &stem.catib6, &suffix.catib6], join_char);
    }
    if db.defines.contains_key("ud") {
        result.ud = utils::join_non_empty(&[&prefix.ud, &stem.ud, &suffix.ud], join_char);
    }
    if db.defines.contains_key("caphi") {
        result.caphi =
            utils::join_non_empty(&[&prefix.caphi, &stem.caphi, &suffix.caphi], join_char);
    }

    let stem_diac = &stem.diac;

    if db.defines.contains_key("d1seg") {
        let s = if stem.d1seg.is_empty() {
            stem_diac
        } else {
            &stem.d1seg
        };
        result.d1seg = format!("{}{}{}", prefix.d1seg, s, suffix.d1seg);
    }
    if db.defines.contains_key("d2seg") {
        let s = if stem.d2seg.is_empty() {
            stem_diac
        } else {
            &stem.d2seg
        };
        result.d2seg = format!("{}{}{}", prefix.d2seg, s, suffix.d2seg);
    }
    if db.defines.contains_key("d3seg") {
        let s = if stem.d3seg.is_empty() {
            stem_diac
        } else {
            &stem.d3seg
        };
        result.d3seg = format!("{}{}{}", prefix.d3seg, s, suffix.d3seg);
    }
    if db.defines.contains_key("atbseg") {
        let s = if stem.atbseg.is_empty() {
            stem_diac
        } else {
            &stem.atbseg
        };
        result.atbseg = format!("{}{}{}", prefix.atbseg, s, suffix.atbseg);
    }
    if db.defines.contains_key("d1tok") {
        let s = if stem.d1tok.is_empty() {
            stem_diac
        } else {
            &stem.d1tok
        };
        result.d1tok = format!("{}{}{}", prefix.d1tok, s, suffix.d1tok);
    }
    if db.defines.contains_key("d2tok") {
        let s = if stem.d2tok.is_empty() {
            stem_diac
        } else {
            &stem.d2tok
        };
        result.d2tok = format!("{}{}{}", prefix.d2tok, s, suffix.d2tok);
    }
    if db.defines.contains_key("d3tok") {
        let s = if stem.d3tok.is_empty() {
            stem_diac
        } else {
            &stem.d3tok
        };
        result.d3tok = format!("{}{}{}", prefix.d3tok, s, suffix.d3tok);
    }
    if db.defines.contains_key("atbtok") {
        let s = if stem.atbtok.is_empty() {
            stem_diac
        } else {
            &stem.atbtok
        };
        result.atbtok = format!("{}{}{}", prefix.atbtok, s, suffix.atbtok);
    }
    if db.defines.contains_key("bwtok") {
        let s = if stem.bwtok.is_empty() {
            stem_diac
        } else {
            &stem.bwtok
        };
        result.bwtok = format!("{}{}{}", prefix.bwtok, s, suffix.bwtok);
    }

    result.stem = stem.diac.clone();
    result.stemgloss = stem.gloss.clone();

    result.diac = utils::normalize_tanwyn(&utils::rewrite_diac(&result.diac), diac_mode);

    if db.defines.contains_key("d1tok") {
        result.d1tok = utils::rewrite_tok_1(&result.d1tok);
    }
    if db.defines.contains_key("d2tok") {
        result.d2tok = utils::rewrite_tok_1(&result.d2tok);
    }
    if db.defines.contains_key("atbtok") {
        result.atbtok = utils::rewrite_tok_1(&result.atbtok);
    }
    if db.defines.contains_key("d1seg") {
        result.d1seg = utils::rewrite_tok_1(&result.d1seg);
    }
    if db.defines.contains_key("d2seg") {
        result.d2seg = utils::rewrite_tok_1(&result.d2seg);
    }
    if db.defines.contains_key("d3seg") {
        result.d3seg = utils::rewrite_tok_1(&result.d3seg);
    }
    if db.defines.contains_key("atbseg") {
        result.atbseg = utils::rewrite_tok_1(&result.atbseg);
    }

    if db.defines.contains_key("d3tok") {
        result.d3tok = utils::rewrite_tok_2(&result.d3tok);
    }

    if db.defines.contains_key("caphi") {
        result.caphi = utils::rewrite_caphi(&result.caphi);
    }

    if db.defines.contains_key("form_gen") && result.r#gen == "-" {
        result.r#gen = result.form_gen.clone();
    }
    if db.defines.contains_key("form_num") && result.num == "-" {
        result.num = result.form_num.clone();
    }

    if db.compute_feats.contains("pattern") {
        let stem_pattern = if stem.pattern.is_empty() {
            &stem.diac
        } else {
            &stem.pattern
        };
        result.pattern = format!("{}{}{}", prefix.diac, stem_pattern, suffix.diac);
        result.pattern = utils::rewrite_pattern(&result.pattern);
    }

    Ok(result)
}

pub fn combine_analyses(
    word_dediac: &str,
    db: &morphology_db::MorphologyDB,
    prefix_analyses: &[ScoredAnalysis],
    stem_analyses: &[ScoredAnalysis],
    suffix_analyses: &[ScoredAnalysis],
) -> Result<VecDeque<ScoredAnalysis>> {
    let mut combined = VecDeque::new();

    for prefix_feats in prefix_analyses {
        let prefix_cat = &prefix_feats.category;

        for stem_feats in stem_analyses {
            let stem_cat = &stem_feats.category;

            let Some(compat_stems) = db.prefix_stem_compat.get(prefix_cat) else {
                continue;
            };
            if !compat_stems.contains(stem_cat) {
                continue;
            }

            for suffix_feats in suffix_analyses {
                let suffix_cat = &suffix_feats.category;

                let dominated = !db.stem_suffix_compat.contains_key(stem_cat)
                    || !db.prefix_suffix_compat.contains_key(prefix_cat)
                    || !db
                        .stem_suffix_compat
                        .get(stem_cat)
                        .map_or(false, |s| s.contains(suffix_cat))
                    || !db
                        .prefix_suffix_compat
                        .get(prefix_cat)
                        .map_or(false, |s| s.contains(suffix_cat));

                if dominated {
                    continue;
                }

                let mut merged =
                    merge_features(db, prefix_feats, stem_feats, suffix_feats, "+", "AF")?;
                merged.stem = stem_feats.diac.clone();
                merged.stemcat = stem_cat.clone();

                let merged_dediac = utils::dediac_ar(&merged.diac)?;
                if word_dediac.replace('\u{0640}', "") != merged_dediac {
                    merged.source = "spvar".to_string();
                }

                combined.push_back(merged);
            }
        }
    }

    Ok(combined)
}

pub fn combine_backoff_analyses(
    stem_str: &str,
    db: &morphology_db::MorphologyDB,
    prefix_analyses: &[ScoredAnalysis],
    stem_analyses: &[ScoredAnalysis],
    suffix_analyses: &[ScoredAnalysis],
    backoff_action: &str,
) -> Result<VecDeque<ScoredAnalysis>> {
    let mut combined = VecDeque::new();

    for prefix_feats in prefix_analyses {
        let prefix_cat = &prefix_feats.category;

        for stem_entry in stem_analyses {
            let stem_cat = &stem_entry.category;

            let Some(compat_stems) = db.prefix_stem_compat.get(prefix_cat) else {
                continue;
            };
            if !compat_stems.contains(stem_cat) {
                continue;
            }

            for suffix_feats in suffix_analyses {
                let suffix_cat = &suffix_feats.category;

                if !db
                    .stem_suffix_compat
                    .get(stem_cat)
                    .map_or(false, |s| s.contains(suffix_cat))
                    || !db
                        .prefix_suffix_compat
                        .get(prefix_cat)
                        .map_or(false, |s| s.contains(suffix_cat))
                {
                    continue;
                }

                if backoff_action == "PROP" && !stem_entry.bw.contains("NOUN_PROP") {
                    continue;
                }

                let mut stem_feats = stem_entry.clone();
                stem_feats.bw = utils::replace_noan(&stem_feats.bw, stem_str);
                stem_feats.diac = utils::replace_noan(&stem_feats.diac, stem_str);
                stem_feats.lex = utils::replace_noan(&stem_feats.lex, stem_str);
                stem_feats.caphi = utils::simple_ar_to_caphi(stem_str);

                let mut merged =
                    merge_features(db, prefix_feats, &stem_feats, suffix_feats, "+", "AF")?;
                merged.stem = stem_feats.diac.clone();
                merged.stemcat = stem_cat.clone();
                merged.source = "backoff".to_string();
                merged.pattern = "backoff".to_string();
                merged.gloss = stem_feats.gloss.clone();

                combined.push_back(merged);
            }
        }
    }

    Ok(combined)
}

pub fn analyze(
    word: &str,
    db: &morphology_db::MorphologyDB,
    strict_digit: bool,
    backoff: &str,
) -> Result<VecDeque<ScoredAnalysis>> {
    let word = word.trim();
    let mut analyses: VecDeque<ScoredAnalysis> = VecDeque::new();
    if word.is_empty() {
        return Ok(analyses);
    }

    let word_dediac = utils::dediac_ar(word)?;
    let word_normalized = utils::normalize_ar(word)?;
    let mut backoff_toks = backoff.clone().split("_");
    let backoff_condition = backoff_toks
        .next()
        .context("invalid backoff condition parsing")?;
    let backoff_action = backoff_toks
        .next()
        .context("invalid backoff action pasring")?;

    if (strict_digit && utils::is_strict_digit(word)) || (!strict_digit && utils::is_digit(word)) {
        let default_from_db = db
            .defaults
            .get("digit")
            .context("unable to get digit default from defaults in morphologyDB")?;

        let mut result = default_from_db.clone();
        result.diac = word.to_string();
        result.stem = word.to_string();
        result.stemgloss = word.to_string();
        result.lex = word.to_string();
        result.bw = word.to_string();
        result.gloss = word.to_string();
        result.source = "digit".to_string();

        result.set_tok_all(word);
        result.set_lex_feats_all("DIGIT");
        result.set_logprobs_all(-99.0);

        if db.defines.contains_key("catib6") {
            result.catib6 = "NOM".to_string();
        }

        if db.defines.contains_key("ud") {
            result.ud = "NOM".to_string();
        }

        if db.defines.contains_key("form_gen") && result.r#gen == "-" {
            result.r#gen = result.form_gen.clone();
        }

        if db.defines.contains_key("form_num") && result.num == "-" {
            result.num = result.form_num.clone();
        }

        analyses.push_back(result);
        return Ok(analyses);
    } else if utils::is_punc(word) {
        let default_from_db = db
            .defaults
            .get("punc")
            .context("unable to get punc default from defaults in morphologyDB")?;

        let mut result = default_from_db.clone();
        result.diac = word.to_string();
        result.stem = word.to_string();
        result.stemgloss = word.to_string();
        result.lex = word.to_string();
        result.bw = word.to_string() + "/PUNC";
        result.source = "punc".to_string();

        result.set_tok_all(word);
        result.set_lex_feats_all("PUNC");
        result.set_logprobs_all(-99.0);

        if db.defines.contains_key("catib6") {
            result.catib6 = "PNX".to_string();
        }

        if db.defines.contains_key("ud") {
            result.ud = "PUNCT".to_string();
        }

        if db.defines.contains_key("form_gen") && result.r#gen == "-" {
            result.r#gen = result.form_gen.clone();
        }

        if db.defines.contains_key("form_num") && result.num == "-" {
            result.num = result.form_num.clone();
        }

        analyses.push_back(result);
        return Ok(analyses);
    } else if !utils::is_ar(word) {
        let default_from_db = db
            .defaults
            .get("latin")
            .context("unable to get latin default from defaults in morphologyDB")?;

        let mut result = default_from_db.clone();
        result.diac = word.to_string();
        result.stem = word.to_string();
        result.stemgloss = word.to_string();
        result.lex = word.to_string();
        result.bw = word.to_string() + "/FOREIGN";
        result.source = "foreign".to_string();

        result.set_tok_all(word);
        result.set_lex_feats_all("FOREIGN");
        result.set_logprobs_all(-99.0);

        if db.defines.contains_key("catib6") {
            result.catib6 = "FOREIGN".to_string();
        }

        if db.defines.contains_key("ud") {
            result.ud = "X".to_string();
        }

        if db.defines.contains_key("form_gen") && result.r#gen == "-" {
            result.r#gen = result.form_gen.clone();
        }

        if db.defines.contains_key("form_num") && result.num == "-" {
            result.num = result.form_num.clone();
        }

        analyses.push_back(result);
        return Ok(analyses);
    } else {
        let word_len = word_normalized.len();
        let char_boundaries: Vec<usize> = word_normalized
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(word_len))
            .collect();

        for &p in &char_boundaries {
            if p > db.max_prefix_size {
                break;
            }
            let prefix = &word_normalized[0..p];
            for &end in &char_boundaries {
                if end <= p {
                    continue;
                }
                let suffix_start = end;
                if word_len - suffix_start > db.max_suffix_size {
                    continue;
                }
                let stem = &word_normalized[p..suffix_start];
                let suffix = &word_normalized[suffix_start..word_len];

                let Some(prefix_entries) = db.prefix_hash.get(prefix) else {
                    continue;
                };
                let Some(suffix_entries) = db.suffix_hash.get(suffix) else {
                    continue;
                };
                let Some(stem_entries) = db.stem_hash.get(stem) else {
                    continue;
                };

                let combined = combine_analyses(
                    &word_dediac,
                    &db,
                    prefix_entries,
                    stem_entries,
                    suffix_entries,
                )?;

                analyses.extend(combined);
            }
        }
    }

    if (backoff_condition == "NOAN" && analyses.is_empty()) || (backoff_condition == "ADD") {
        let word_len = word_normalized.len();
        let backoff_cats = db
            .stem_backoffs
            .get(backoff_action)
            .context("could not find backoff action")?;

        let backoff_stems: Vec<ScoredAnalysis> = db
            .stem_hash
            .get("NOAN")
            .map(|entries| {
                entries
                    .iter()
                    .filter(|a| backoff_cats.contains(&a.category))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        if !backoff_stems.is_empty() {
            let char_boundaries: Vec<usize> = word_normalized
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(word_len))
                .collect();

            for &p in &char_boundaries {
                if p > db.max_prefix_size {
                    break;
                }
                let prefix = &word_normalized[0..p];
                for &end in &char_boundaries {
                    if end <= p {
                        continue;
                    }
                    let suffix_start = end;
                    if word_len - suffix_start > db.max_suffix_size {
                        continue;
                    }
                    let stem_str = &word_normalized[p..suffix_start];
                    let suffix = &word_normalized[suffix_start..word_len];

                    let Some(prefix_entries) = db.prefix_hash.get(prefix) else {
                        continue;
                    };
                    let Some(suffix_entries) = db.suffix_hash.get(suffix) else {
                        continue;
                    };

                    let combined = combine_backoff_analyses(
                        stem_str,
                        &db,
                        prefix_entries,
                        &backoff_stems,
                        suffix_entries,
                        backoff_action,
                    )?;

                    analyses.extend(combined);
                }
            }
        }
    }

    Ok(analyses)
}

pub fn analyze_words(
    words: &[&str],
    db: &morphology_db::MorphologyDB,
) -> Result<Vec<VecDeque<ScoredAnalysis>>> {
    words
        .iter()
        .map(|w| analyze(w, db, true, "NOAN_PROP"))
        .collect()
}
