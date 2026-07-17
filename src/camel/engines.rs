use crate::camel::analyzer::{self, ScoredAnalysis};
use crate::camel::mle;
use crate::camel::morphology_db::MorphologyDB;
use crate::camel::resources::{self, CamelResources};
use crate::camel::stemmer;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AnalyzerEngine {
    db: MorphologyDB,
    backoff: String,
    strict_digit: bool,
}

impl AnalyzerEngine {
    pub fn load(name: &str, backoff: &str, strict_digit: bool) -> Result<Self> {
        Self::load_with_download(name, backoff, strict_digit, true)
    }

    pub fn load_with_download(
        name: &str,
        backoff: &str,
        strict_digit: bool,
        allow_download: bool,
    ) -> Result<Self> {
        Ok(Self {
            db: resources::load_morphology_db_with_download(name, allow_download)?,
            backoff: backoff.to_owned(),
            strict_digit,
        })
    }

    pub fn analyze(&self, word: &str) -> Result<Vec<ScoredAnalysis>> {
        Ok(
            analyzer::analyze(word, &self.db, self.strict_digit, &self.backoff)?
                .into_iter()
                .collect(),
        )
    }
}

pub struct MleEngine {
    resources: Arc<CamelResources>,
}

impl MleEngine {
    pub fn load(name: &str) -> Result<Self> {
        Self::load_with_download(name, true)
    }

    pub fn load_with_download(name: &str, allow_download: bool) -> Result<Self> {
        Ok(Self {
            resources: Arc::new(CamelResources::load_with_download(name, allow_download)?),
        })
    }

    pub fn disambiguate(
        &self,
        words: &[String],
        backoff: &str,
        top: usize,
    ) -> Result<Vec<Vec<ScoredAnalysis>>> {
        let words: Vec<&str> = words.iter().map(String::as_str).collect();
        mle::disambiguate(
            &words,
            &self.resources.db,
            &self.resources.model,
            backoff,
            top,
        )
    }
}

pub struct StemmerEngine {
    resources: Arc<CamelResources>,
    cache: Option<Mutex<HashMap<(String, String), Vec<ScoredAnalysis>>>>,
    max_cache_size: usize,
}

impl StemmerEngine {
    pub fn load(name: &str, max_cache_size: usize) -> Result<Self> {
        Self::load_with_download(name, max_cache_size, true)
    }

    pub fn load_with_download(
        name: &str,
        max_cache_size: usize,
        allow_download: bool,
    ) -> Result<Self> {
        Ok(Self {
            resources: Arc::new(CamelResources::load_with_download(name, allow_download)?),
            cache: (max_cache_size > 0).then(|| Mutex::new(HashMap::new())),
            max_cache_size,
        })
    }

    pub fn stem(
        &self,
        text: &str,
        sep: &str,
        scheme: &str,
        preserve_diacritics: bool,
        backoff: &str,
        fallback: &[String],
    ) -> Result<String> {
        let keys = stemmer::cache_keys(text);
        let mut request_cache = match &self.cache {
            Some(cache) => {
                let cache = cache
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                keys.iter()
                    .filter_map(|key| {
                        cache
                            .get(&(backoff.to_owned(), key.clone()))
                            .cloned()
                            .map(|value| (key.clone(), value))
                    })
                    .collect()
            }
            None => HashMap::new(),
        };
        let fallback: Vec<&str> = fallback.iter().map(String::as_str).collect();
        let result = stemmer::stem(
            text,
            &self.resources.db,
            &self.resources.model,
            sep,
            scheme,
            preserve_diacritics,
            backoff,
            Some(&mut request_cache),
            0,
            &fallback,
        )?;

        if let Some(cache) = &self.cache {
            let mut cache = cache
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if cache.len().saturating_add(request_cache.len()) > self.max_cache_size {
                cache.clear();
            }
            for (key, value) in request_cache.into_iter().take(self.max_cache_size) {
                cache.insert((backoff.to_owned(), key), value);
            }
        }

        Ok(result)
    }

    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
    }

    pub fn cache_size(&self) -> usize {
        self.cache.as_ref().map_or(0, |cache| {
            cache
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .len()
        })
    }
}
