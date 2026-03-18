use crate::camel::analyzer::ScoredAnalysis;
use crate::camel::supplement_stems;
use crate::utils;
use anyhow::Context;
use anyhow::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct MorphologyDB {
    pub defines: HashMap<String, Option<Vec<String>>>,
    pub defaults: HashMap<String, ScoredAnalysis>,
    pub order: Vec<String>,
    pub tokenizations: HashSet<String>,
    pub compute_feats: HashSet<String>,
    pub stem_backoffs: HashMap<String, Vec<String>>,
    pub prefix_hash: HashMap<String, Vec<ScoredAnalysis>>,
    pub suffix_hash: HashMap<String, Vec<ScoredAnalysis>>,
    pub stem_hash: HashMap<String, Vec<ScoredAnalysis>>,
    pub prefix_cat_hash: HashMap<String, String>,
    pub suffix_cat_hash: HashMap<String, String>,
    pub lemma_hash: HashMap<String, String>,
    pub prefix_stem_compat: HashMap<String, HashSet<String>>,
    pub stem_suffix_compat: HashMap<String, HashSet<String>>,
    pub prefix_suffix_compat: HashMap<String, HashSet<String>>,
    _stem_prefix_compat: HashMap<String, HashSet<String>>,
    pub max_prefix_size: usize,
    pub max_suffix_size: usize,
}

impl MorphologyDB {
    pub fn new() -> Self {
        MorphologyDB {
            defines: HashMap::new(),
            defaults: HashMap::new(),
            order: Vec::new(),
            tokenizations: HashSet::new(),
            compute_feats: HashSet::new(),
            stem_backoffs: HashMap::new(),
            prefix_hash: HashMap::new(),
            suffix_hash: HashMap::new(),
            stem_hash: HashMap::new(),
            prefix_cat_hash: HashMap::new(),
            suffix_cat_hash: HashMap::new(),
            lemma_hash: HashMap::new(),
            prefix_stem_compat: HashMap::new(),
            stem_suffix_compat: HashMap::new(),
            prefix_suffix_compat: HashMap::new(),
            _stem_prefix_compat: HashMap::new(),
            max_prefix_size: 0,
            max_suffix_size: 0,
        }
    }

    pub fn load(dbpath: PathBuf) -> Result<Self> {
        let mut db = Self::new();
        let dbpath = if dbpath.is_dir() {
            dbpath.join("morphology.db")
        } else {
            dbpath
        };
        let dbfile = fs::read_to_string(&dbpath).context(format!(
            "Failed to read morphology db at {}",
            dbpath.display()
        ))?;
        let db_lines = dbfile.lines();
        let mut current_section = "";
        for line in db_lines {
            if line.starts_with("###") {
                current_section = &line[3..line.len() - 3];
                continue;
            }
            match current_section {
                "DEFINES" => {
                    let mut line_els = line.splitn(2, " ");
                    line_els.next().context("DEFINE KEYWORD DUMPING")?;

                    let mut define_type_value = line_els
                        .next()
                        .context("DEFINE type and value not existant in db line")?
                        .splitn(2, " ");
                    let define_type = define_type_value
                        .next()
                        .context("DEFINE type not existant in db line")?;
                    let define_values = define_type_value
                        .next()
                        .context("DEFINE value not existant in db line")?
                        .split(" ");

                    let mut define_value: Option<Vec<String>> = Some(Vec::new());
                    for def_val in define_values {
                        let mut comp = def_val.splitn(2, ":");
                        let curr_type = comp
                            .next()
                            .context("malformed define value when splitting by :")?;

                        if curr_type != define_type {
                            break;
                        }

                        let token_type = comp
                            .next()
                            .context("malformed define value when splitting by :")?;
                        if token_type == "*open*" {
                            define_value = None;
                            break;
                        }

                        define_value
                            .as_mut()
                            .expect("define_value should be initialized")
                            .push(token_type.to_string());
                    }

                    db.defines.insert(define_type.to_string(), define_value);
                }

                "ORDER" => {
                    let mut line_els = line.splitn(2, " ");
                    line_els.next().context("ORDER KEYWORD DUMPING")?;

                    let order_line_value = line_els
                        .next()
                        .context("order key-value pairs not present in db line")?
                        .split(" ");

                    for olv in order_line_value {
                        db.order.push(olv.to_string());
                    }
                }

                "TOKENIZATIONS" => {
                    let mut line_els = line.splitn(2, " ");

                    line_els.next().context("TOKENIZATION KEYWORD DUMPING")?;
                    db.compute_feats = HashSet::from_iter(db.order.iter().cloned());
                    let tokenization_line_value = line_els
                        .next()
                        .context("tokenization key-value pairs not present in db line")?
                        .split(" ");

                    for tlv in tokenization_line_value {
                        db.tokenizations.insert(tlv.to_string());
                    }
                }

                "STEMBACKOFF" => {
                    let mut line_els = line.splitn(2, " ");

                    line_els.next().context("STEMBACKOFF KEYWORD DUMPING")?;
                    let mut stembackoff_line_value = line_els
                        .next()
                        .context("stembackoff key-value pairs not present in db line")?
                        .splitn(2, " ");

                    let stem_backoff_key = stembackoff_line_value
                        .next()
                        .context("stembackoff key within line not present in line")?;

                    let stem_backoff_values = stembackoff_line_value
                        .next()
                        .context("stembackoff values not within line")?
                        .split(" ");

                    let mut stem_backoff_values_vec = Vec::new();

                    for sbv in stem_backoff_values {
                        stem_backoff_values_vec.push(sbv.to_string());
                    }

                    db.stem_backoffs
                        .insert(stem_backoff_key.to_string(), stem_backoff_values_vec);
                }

                "DEFAULTS" => {
                    let mut line_els = line.splitn(2, " ");

                    line_els.next().context("DEFAULT KEYWORD DUMPING")?;
                    let default_line_value = line_els
                        .next()
                        .context("default key-value pairs not present in db line")?
                        .split(" ");

                    let mut default_line_els: HashMap<String, String> = HashMap::new();
                    for def_val in default_line_value {
                        let mut comp = def_val.splitn(2, ":");
                        let comp_key = comp.next().context("default comp key not valid")?;
                        let comp_val = comp.next().context("default comp val not valid")?;
                        default_line_els.insert(comp_key.to_string(), comp_val.to_string());
                    }

                    if let Some(default_key) = default_line_els.get("pos").filter(|v| *v != "*") {
                        let scored = ScoredAnalysis::from_map(&default_line_els)?;
                        db.defaults.insert(default_key.clone(), scored);
                    }
                }

                "PREFIXES" => {
                    let mut line_els = line.split("\t");
                    let prefix = line_els
                        .next()
                        .context("not able to get prefix from prefix line")?
                        .trim();
                    let category = line_els
                        .next()
                        .context("not able to get category from prefix line")?;
                    let analysis_els = line_els
                        .next()
                        .context("not able to get analysis elements from prefix line")?
                        .split(" ");

                    let mut analysis_map: HashMap<String, String> = HashMap::new();

                    for ael in analysis_els {
                        if ael.len() == 0 {
                            continue;
                        }
                        let mut ael_spl = ael.splitn(2, ":");

                        analysis_map.insert(
                            ael_spl
                                .next()
                                .context("splitting prefix analysis 1 not working")?
                                .to_string(),
                            ael_spl
                                .next()
                                .context("splitting prefix analysis 2 not working")?
                                .to_string(),
                        );
                    }

                    let mut scored = ScoredAnalysis::from_map(&analysis_map)?;
                    scored.category = category.to_string();
                    db.prefix_hash
                        .entry(prefix.to_string())
                        .or_default()
                        .push(scored);
                }
                "SUFFIXES" => {
                    let mut line_els = line.split("\t");
                    let suffix = line_els
                        .next()
                        .context("not able to get suffix from suffix line")?
                        .trim();
                    let category = line_els
                        .next()
                        .context("not able to get category from suffix line")?;
                    let analysis_els = line_els
                        .next()
                        .context("not able to get analysis elements from suffix line")?
                        .split(" ");

                    let mut analysis_map: HashMap<String, String> = HashMap::new();

                    for ael in analysis_els {
                        if ael.len() == 0 {
                            continue;
                        }
                        let mut ael_spl = ael.splitn(2, ":");
                        analysis_map.insert(
                            ael_spl
                                .next()
                                .context("splitting suffix analysis 1 not working")?
                                .to_string(),
                            ael_spl
                                .next()
                                .context("splitting suffix analysis 2 not working")?
                                .to_string(),
                        );
                    }

                    let mut scored = ScoredAnalysis::from_map(&analysis_map)?;
                    scored.category = category.to_string();
                    db.suffix_hash
                        .entry(suffix.to_string())
                        .or_default()
                        .push(scored);
                }
                "STEMS" => {
                    let mut line_els = line.split("\t");
                    let stem = line_els
                        .next()
                        .context("not able to get stem from stem line")?
                        .trim();
                    let category = line_els
                        .next()
                        .context("not able to get category from stem line")?;
                    let analysis_els = line_els
                        .next()
                        .context("not able to get analysis elements from stem line")?
                        .split(" ");

                    let mut analysis_map: HashMap<String, String> = HashMap::new();

                    for ael in analysis_els {
                        if ael.len() == 0 {
                            continue;
                        }
                        let mut ael_spl = ael.splitn(2, ":");
                        analysis_map.insert(
                            ael_spl
                                .next()
                                .context("splitting stem analysis 1 not working")?
                                .to_string(),
                            ael_spl
                                .next()
                                .context("splitting stem analysis 2 not working")?
                                .to_string(),
                        );
                    }
                    // Strip the lex value before deserializing
                    if let Some(lex_val) = analysis_map.get("lex") {
                        let stripped = utils::strip_lex(lex_val)?;
                        analysis_map.insert("lex".to_string(), stripped);
                    }

                    let mut scored = ScoredAnalysis::from_map(&analysis_map)?;
                    scored.category = category.to_string();
                    db.stem_hash
                        .entry(stem.to_string())
                        .or_default()
                        .push(scored);
                }
                "TABLE AB" => {
                    let mut line_els = line.split(" ");
                    let prefix_cat = line_els
                        .next()
                        .context("not able to get prefix cat from TABLE AB line")?
                        .trim();
                    let stem_cat = line_els
                        .next()
                        .context("not able to get stem cat from TABLE AB line")?;

                    db.prefix_stem_compat
                        .entry(prefix_cat.to_string())
                        .or_default()
                        .insert(stem_cat.to_string());
                }
                "TABLE BC" => {
                    let mut line_els = line.split(" ");
                    let stem_cat = line_els
                        .next()
                        .context("not able to get stem cat from TABLE BC line")?
                        .trim();
                    let suffix_cat = line_els
                        .next()
                        .context("not able to get suffix cat from TABLE BC line")?;

                    db.stem_suffix_compat
                        .entry(stem_cat.to_string())
                        .or_default()
                        .insert(suffix_cat.to_string());
                }
                "TABLE AC" => {
                    let mut line_els = line.split(" ");
                    let prefix_cat = line_els
                        .next()
                        .context("not able to get prefix cat from TABLE AC line")?
                        .trim();
                    let suffix_cat = line_els
                        .next()
                        .context("not able to get suffix cat from TABLE AC line")?;

                    db.prefix_suffix_compat
                        .entry(prefix_cat.to_string())
                        .or_default()
                        .insert(suffix_cat.to_string());
                }

                _ => continue,
            }
        }

        db.max_prefix_size = db.prefix_hash.keys().map(String::len).max().unwrap_or(0);
        db.max_suffix_size = db.suffix_hash.keys().map(String::len).max().unwrap_or(0);

        db.inject_supplementary_stems(
            supplement_stems::SUPPLEMENT_AL_T,
            supplement_stems::SUPPLEMENT_AL,
            supplement_stems::SUPPLEMENT_T,
        );

        Ok(db)
    }

    fn make_nall_stem(surface: &str) -> ScoredAnalysis {
        ScoredAnalysis {
            category: "Nall".to_string(),
            diac: surface.to_string(),
            lex: surface.to_string(),
            bw: format!("{}/NOUN", surface),
            gloss: "NO_GLOSS".to_string(),
            pos: "noun".to_string(),
            per: "na".to_string(),
            asp: "na".to_string(),
            vox: "na".to_string(),
            r#mod: "na".to_string(),
            r#gen: "-".to_string(),
            num: "-".to_string(),
            stt: "i".to_string(),
            cas: "u".to_string(),
            rat: "i".to_string(),
            enc0: "0".to_string(),
            prc0: "0".to_string(),
            prc1: "0".to_string(),
            prc2: "0".to_string(),
            prc3: "0".to_string(),
            form_gen: "m".to_string(),
            form_num: "s".to_string(),
            source: "lex".to_string(),
            catib6: "NOM".to_string(),
            ud: "NOUN".to_string(),
            d1seg: surface.to_string(),
            d2seg: surface.to_string(),
            d3seg: surface.to_string(),
            atbseg: surface.to_string(),
            d1tok: surface.to_string(),
            d2tok: surface.to_string(),
            d3tok: surface.to_string(),
            atbtok: surface.to_string(),
            bwtok: surface.to_string(),
            pos_logprob: -99.0,
            lex_logprob: -99.0,
            pos_lex_logprob: -99.0,
            ..Default::default()
        }
    }

    pub fn inject_supplementary_stems(
        &mut self,
        al_t_words: &[&str],
        al_words: &[&str],
        t_words: &[&str],
    ) {
        let al_len: usize = "ال".len();
        let haa_len: usize = "ه".len();

        for word in al_t_words {
            let normalized = utils::normalize_ar(word).unwrap_or_else(|_| word.to_string());
            if normalized.len() <= al_len + haa_len {
                continue;
            }
            let stem = &normalized[al_len..normalized.len() - haa_len];
            self.stem_hash
                .entry(stem.to_string())
                .or_default()
                .push(Self::make_nall_stem(stem));
        }

        for word in al_words {
            let normalized = utils::normalize_ar(word).unwrap_or_else(|_| word.to_string());
            if normalized.len() <= al_len {
                continue;
            }
            let stem = &normalized[al_len..];
            self.stem_hash
                .entry(stem.to_string())
                .or_default()
                .push(Self::make_nall_stem(stem));
        }

        for word in t_words {
            let normalized = utils::normalize_ar(word).unwrap_or_else(|_| word.to_string());
            if normalized.len() <= haa_len {
                continue;
            }
            let stem = &normalized[..normalized.len() - haa_len];
            self.stem_hash
                .entry(stem.to_string())
                .or_default()
                .push(Self::make_nall_stem(stem));
        }
    }
}
