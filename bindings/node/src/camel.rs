use fast_disambig::camel::analyzer::ScoredAnalysis;
use fast_disambig::camel::downloader;
use fast_disambig::camel::engines::{AnalyzerEngine, MleEngine, StemmerEngine};
use fast_disambig::utils;
use napi::bindgen_prelude::{AsyncTask, Env, Task};
use napi::{Error, Result, Status};
use std::sync::Arc;

fn napi_error(error: impl std::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, error.to_string())
}

#[napi(
    object,
    object_from_js = false,
    namespace = "camel",
    js_name = "ScoredAnalysis"
)]
pub struct JsScoredAnalysis {
    pub score: f64,
    pub category: String,
    pub source: String,
    pub diac: String,
    pub lex: String,
    pub bw: String,
    pub gloss: String,
    pub stem: String,
    pub stemcat: String,
    pub stemgloss: String,
    pub catib6: String,
    pub ud: String,
    pub per: String,
    pub asp: String,
    pub vox: String,
    #[napi(js_name = "mod")]
    pub modifier: String,
    #[napi(js_name = "gen")]
    pub gender: String,
    pub num: String,
    pub stt: String,
    pub cas: String,
    pub rat: String,
    pub form_gen: String,
    pub form_num: String,
    pub pos: String,
    pub prc3: String,
    pub prc2: String,
    pub prc1: String,
    pub prc0: String,
    pub enc0: String,
    pub enc1: String,
    pub enc2: String,
    #[napi(js_name = "d1seg")]
    pub d1seg: String,
    #[napi(js_name = "d2seg")]
    pub d2seg: String,
    #[napi(js_name = "d3seg")]
    pub d3seg: String,
    pub atbseg: String,
    #[napi(js_name = "d1tok")]
    pub d1tok: String,
    #[napi(js_name = "d2tok")]
    pub d2tok: String,
    #[napi(js_name = "d3tok")]
    pub d3tok: String,
    pub atbtok: String,
    pub bwtok: String,
    pub root: String,
    pub pattern: String,
    pub caphi: String,
    pub pos_logprob: f64,
    pub lex_logprob: f64,
    pub pos_lex_logprob: f64,
}

impl From<ScoredAnalysis> for JsScoredAnalysis {
    fn from(value: ScoredAnalysis) -> Self {
        Self {
            score: value.score,
            category: value.category,
            source: value.source,
            diac: value.diac,
            lex: value.lex,
            bw: value.bw,
            gloss: value.gloss,
            stem: value.stem,
            stemcat: value.stemcat,
            stemgloss: value.stemgloss,
            catib6: value.catib6,
            ud: value.ud,
            per: value.per,
            asp: value.asp,
            vox: value.vox,
            modifier: value.r#mod,
            gender: value.r#gen,
            num: value.num,
            stt: value.stt,
            cas: value.cas,
            rat: value.rat,
            form_gen: value.form_gen,
            form_num: value.form_num,
            pos: value.pos,
            prc3: value.prc3,
            prc2: value.prc2,
            prc1: value.prc1,
            prc0: value.prc0,
            enc0: value.enc0,
            enc1: value.enc1,
            enc2: value.enc2,
            d1seg: value.d1seg,
            d2seg: value.d2seg,
            d3seg: value.d3seg,
            atbseg: value.atbseg,
            d1tok: value.d1tok,
            d2tok: value.d2tok,
            d3tok: value.d3tok,
            atbtok: value.atbtok,
            bwtok: value.bwtok,
            root: value.root,
            pattern: value.pattern,
            caphi: value.caphi,
            pos_logprob: value.pos_logprob,
            lex_logprob: value.lex_logprob,
            pos_lex_logprob: value.pos_lex_logprob,
        }
    }
}

#[napi(
    object,
    object_from_js = false,
    namespace = "camel",
    js_name = "DisambiguatedWord"
)]
pub struct JsDisambiguatedWord {
    pub word: String,
    pub analyses: Vec<JsScoredAnalysis>,
}

fn disambiguated_words(
    words: Vec<String>,
    analyses: Vec<Vec<ScoredAnalysis>>,
) -> Vec<JsDisambiguatedWord> {
    words
        .into_iter()
        .zip(analyses)
        .map(|(word, analyses)| JsDisambiguatedWord {
            word,
            analyses: analyses.into_iter().map(Into::into).collect(),
        })
        .collect()
}

#[napi(object, namespace = "camel")]
pub struct StemmerOptions {
    pub name: Option<String>,
    pub cache_size: Option<u32>,
    pub allow_download: Option<bool>,
}

#[napi(object, namespace = "camel")]
pub struct StemOptions {
    pub sep: Option<String>,
    pub scheme: Option<String>,
    pub preserve_diacritics: Option<bool>,
    pub backoff: Option<String>,
    pub fallback: Option<Vec<String>>,
}

struct ResolvedStemOptions {
    sep: String,
    scheme: String,
    preserve_diacritics: bool,
    backoff: String,
    fallback: Vec<String>,
}

impl From<Option<StemOptions>> for ResolvedStemOptions {
    fn from(options: Option<StemOptions>) -> Self {
        let options = options.unwrap_or(StemOptions {
            sep: None,
            scheme: None,
            preserve_diacritics: None,
            backoff: None,
            fallback: None,
        });
        Self {
            sep: options.sep.unwrap_or_else(|| "[+]".to_owned()),
            scheme: options.scheme.unwrap_or_else(|| "d3tok".to_owned()),
            preserve_diacritics: options.preserve_diacritics.unwrap_or(false),
            backoff: options.backoff.unwrap_or_else(|| "NOAN_PROP".to_owned()),
            fallback: options.fallback.unwrap_or_default(),
        }
    }
}

fn stemmer_config(options: Option<StemmerOptions>) -> (String, usize, bool) {
    let options = options.unwrap_or(StemmerOptions {
        name: None,
        cache_size: None,
        allow_download: None,
    });
    (
        options.name.unwrap_or_else(|| "calima-msa-r13".to_owned()),
        options.cache_size.unwrap_or(100_000) as usize,
        options.allow_download.unwrap_or(true),
    )
}

#[napi(namespace = "camel", js_name = "Stemmer")]
pub struct JsStemmer {
    inner: Arc<StemmerEngine>,
}

#[napi(namespace = "camel")]
impl JsStemmer {
    #[napi(constructor)]
    pub fn new(options: Option<StemmerOptions>) -> Result<Self> {
        let (name, cache_size, allow_download) = stemmer_config(options);
        Ok(Self {
            inner: Arc::new(
                StemmerEngine::load_with_download(&name, cache_size, allow_download)
                    .map_err(napi_error)?,
            ),
        })
    }

    #[napi(factory)]
    pub async fn create(options: Option<StemmerOptions>) -> Result<Self> {
        let (name, cache_size, allow_download) = stemmer_config(options);
        let inner = napi::tokio::task::spawn_blocking(move || {
            StemmerEngine::load_with_download(&name, cache_size, allow_download)
        })
        .await
        .map_err(napi_error)?
        .map_err(napi_error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[napi(ts_return_type = "Promise<string>")]
    pub fn stem(&self, text: String, options: Option<StemOptions>) -> AsyncTask<StemTask> {
        AsyncTask::new(StemTask {
            engine: Arc::clone(&self.inner),
            text,
            options: options.into(),
        })
    }

    #[napi]
    pub fn stem_sync(&self, text: String, options: Option<StemOptions>) -> Result<String> {
        let options: ResolvedStemOptions = options.into();
        self.inner
            .stem(
                &text,
                &options.sep,
                &options.scheme,
                options.preserve_diacritics,
                &options.backoff,
                &options.fallback,
            )
            .map_err(napi_error)
    }

    #[napi]
    pub fn clear_cache(&self) {
        self.inner.clear_cache();
    }

    #[napi(getter)]
    pub fn cache_size(&self) -> Result<u32> {
        self.inner
            .cache_size()
            .try_into()
            .map_err(|_| napi_error("cache size exceeds the JavaScript number boundary"))
    }
}

pub struct StemTask {
    engine: Arc<StemmerEngine>,
    text: String,
    options: ResolvedStemOptions,
}

impl Task for StemTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        self.engine
            .stem(
                &self.text,
                &self.options.sep,
                &self.options.scheme,
                self.options.preserve_diacritics,
                &self.options.backoff,
                &self.options.fallback,
            )
            .map_err(napi_error)
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi(object, namespace = "camel")]
pub struct MleOptions {
    pub name: Option<String>,
    pub allow_download: Option<bool>,
}

#[napi(object, namespace = "camel")]
pub struct DisambiguateOptions {
    pub backoff: Option<String>,
    pub top: Option<u32>,
}

fn model_config(options: Option<MleOptions>) -> (String, bool) {
    let options = options.unwrap_or(MleOptions {
        name: None,
        allow_download: None,
    });
    (
        options.name.unwrap_or_else(|| "calima-msa-r13".to_owned()),
        options.allow_download.unwrap_or(true),
    )
}

fn disambiguate_config(options: Option<DisambiguateOptions>) -> (String, usize) {
    let options = options.unwrap_or(DisambiguateOptions {
        backoff: None,
        top: None,
    });
    (
        options.backoff.unwrap_or_else(|| "NOAN_PROP".to_owned()),
        options.top.unwrap_or(1) as usize,
    )
}

#[napi(namespace = "camel", js_name = "MLEDisambiguator")]
pub struct JsMleDisambiguator {
    inner: Arc<MleEngine>,
}

#[napi(namespace = "camel")]
impl JsMleDisambiguator {
    #[napi(constructor)]
    pub fn new(options: Option<MleOptions>) -> Result<Self> {
        let (name, allow_download) = model_config(options);
        Ok(Self {
            inner: Arc::new(
                MleEngine::load_with_download(&name, allow_download).map_err(napi_error)?,
            ),
        })
    }

    #[napi(factory)]
    pub async fn create(options: Option<MleOptions>) -> Result<Self> {
        let (name, allow_download) = model_config(options);
        let inner = napi::tokio::task::spawn_blocking(move || {
            MleEngine::load_with_download(&name, allow_download)
        })
        .await
        .map_err(napi_error)?
        .map_err(napi_error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[napi(ts_return_type = "Promise<Array<DisambiguatedWord>>")]
    pub fn disambiguate(
        &self,
        words: Vec<String>,
        options: Option<DisambiguateOptions>,
    ) -> AsyncTask<DisambiguateTask> {
        let (backoff, top) = disambiguate_config(options);
        AsyncTask::new(DisambiguateTask {
            engine: Arc::clone(&self.inner),
            words,
            backoff,
            top,
        })
    }

    #[napi]
    pub fn disambiguate_sync(
        &self,
        words: Vec<String>,
        options: Option<DisambiguateOptions>,
    ) -> Result<Vec<JsDisambiguatedWord>> {
        let (backoff, top) = disambiguate_config(options);
        let analyses = self
            .inner
            .disambiguate(&words, &backoff, top)
            .map_err(napi_error)?;
        Ok(disambiguated_words(words, analyses))
    }
}

pub struct DisambiguateTask {
    engine: Arc<MleEngine>,
    words: Vec<String>,
    backoff: String,
    top: usize,
}

impl Task for DisambiguateTask {
    type Output = Vec<Vec<ScoredAnalysis>>;
    type JsValue = Vec<JsDisambiguatedWord>;

    fn compute(&mut self) -> Result<Self::Output> {
        self.engine
            .disambiguate(&self.words, &self.backoff, self.top)
            .map_err(napi_error)
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(disambiguated_words(self.words.clone(), output))
    }
}

#[napi(object, namespace = "camel")]
pub struct AnalyzerOptions {
    pub name: Option<String>,
    pub backoff: Option<String>,
    pub strict_digit: Option<bool>,
    pub allow_download: Option<bool>,
}

fn analyzer_config(options: Option<AnalyzerOptions>) -> (String, String, bool, bool) {
    let options = options.unwrap_or(AnalyzerOptions {
        name: None,
        backoff: None,
        strict_digit: None,
        allow_download: None,
    });
    (
        options.name.unwrap_or_else(|| "calima-msa-r13".to_owned()),
        options.backoff.unwrap_or_else(|| "NOAN_PROP".to_owned()),
        options.strict_digit.unwrap_or(false),
        options.allow_download.unwrap_or(true),
    )
}

#[napi(namespace = "camel", js_name = "Analyzer")]
pub struct JsAnalyzer {
    inner: Arc<AnalyzerEngine>,
}

#[napi(namespace = "camel")]
impl JsAnalyzer {
    #[napi(constructor)]
    pub fn new(options: Option<AnalyzerOptions>) -> Result<Self> {
        let (name, backoff, strict_digit, allow_download) = analyzer_config(options);
        Ok(Self {
            inner: Arc::new(
                AnalyzerEngine::load_with_download(&name, &backoff, strict_digit, allow_download)
                    .map_err(napi_error)?,
            ),
        })
    }

    #[napi(factory)]
    pub async fn create(options: Option<AnalyzerOptions>) -> Result<Self> {
        let (name, backoff, strict_digit, allow_download) = analyzer_config(options);
        let inner = napi::tokio::task::spawn_blocking(move || {
            AnalyzerEngine::load_with_download(&name, &backoff, strict_digit, allow_download)
        })
        .await
        .map_err(napi_error)?
        .map_err(napi_error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[napi(ts_return_type = "Promise<Array<ScoredAnalysis>>")]
    pub fn analyze(&self, word: String) -> AsyncTask<AnalyzeTask> {
        AsyncTask::new(AnalyzeTask {
            engine: Arc::clone(&self.inner),
            word,
        })
    }

    #[napi]
    pub fn analyze_sync(&self, word: String) -> Result<Vec<JsScoredAnalysis>> {
        self.inner
            .analyze(&word)
            .map(|analyses| analyses.into_iter().map(Into::into).collect())
            .map_err(napi_error)
    }
}

pub struct AnalyzeTask {
    engine: Arc<AnalyzerEngine>,
    word: String,
}

impl Task for AnalyzeTask {
    type Output = Vec<ScoredAnalysis>;
    type JsValue = Vec<JsScoredAnalysis>;

    fn compute(&mut self) -> Result<Self::Output> {
        self.engine.analyze(&self.word).map_err(napi_error)
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output.into_iter().map(Into::into).collect())
    }
}

#[napi(namespace = "camel")]
pub fn tokenize(sentence: String, mode: Option<String>) -> Vec<String> {
    utils::simple_word_tokenize(&sentence, mode.as_deref().unwrap_or("compact"))
}

#[napi(namespace = "camel", js_name = "dediacAr")]
pub fn dediac_ar(text: String) -> Result<String> {
    utils::dediac_ar(&text).map_err(napi_error)
}

#[napi(namespace = "camel", js_name = "dataDir")]
pub fn data_dir() -> Result<String> {
    downloader::get_or_create_camel_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(napi_error)
}
