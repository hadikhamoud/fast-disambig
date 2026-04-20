use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::analyzer;
use crate::analyzer::ScoredAnalysis;
use crate::downloader;
use crate::mle;
use crate::morphology_db::MorphologyDB;
use crate::stemmer;
use crate::utils;

#[pyclass]
pub struct PyMorphologyDB {
    inner: MorphologyDB,
}

#[pymethods]
impl PyMorphologyDB {
    #[new]
    fn new(path_or_name: String) -> PyResult<Self> {
        let db = MorphologyDB::load(resolve_resource_path(&path_or_name, &["MorphologyDB"])?)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyMorphologyDB { inner: db })
    }

    #[staticmethod]
    fn builtin_db(name: &str) -> PyResult<Self> {
        let db_path = resolve_resource_path(name, &["MorphologyDB"])?;
        let db = MorphologyDB::load(db_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyMorphologyDB { inner: db })
    }
}

#[pyclass]
pub struct PyMLEModel {
    inner: HashMap<String, ScoredAnalysis>,
}

#[pymethods]
impl PyMLEModel {
    #[new]
    fn new(path_or_name: String) -> PyResult<Self> {
        let model = mle::load_mle_model(resolve_resource_path(&path_or_name, &["DisambigMLE"])?)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyMLEModel { inner: model })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyScoredAnalysis {
    inner: ScoredAnalysis,
}

#[pymethods]
impl PyScoredAnalysis {
    #[getter]
    fn score(&self) -> f64 {
        self.inner.score
    }
    #[getter]
    fn diac(&self) -> &str {
        &self.inner.diac
    }
    #[getter]
    fn lex(&self) -> &str {
        &self.inner.lex
    }
    #[getter]
    fn bw(&self) -> &str {
        &self.inner.bw
    }
    #[getter]
    fn gloss(&self) -> &str {
        &self.inner.gloss
    }
    #[getter]
    fn pos(&self) -> &str {
        &self.inner.pos
    }
    #[getter]
    fn stem(&self) -> &str {
        &self.inner.stem
    }
    #[getter]
    fn stemcat(&self) -> &str {
        &self.inner.stemcat
    }
    #[getter]
    fn stemgloss(&self) -> &str {
        &self.inner.stemgloss
    }
    #[getter]
    fn source(&self) -> &str {
        &self.inner.source
    }
    #[getter]
    fn catib6(&self) -> &str {
        &self.inner.catib6
    }
    #[getter]
    fn ud(&self) -> &str {
        &self.inner.ud
    }
    #[getter]
    fn per(&self) -> &str {
        &self.inner.per
    }
    #[getter]
    fn asp(&self) -> &str {
        &self.inner.asp
    }
    #[getter]
    fn vox(&self) -> &str {
        &self.inner.vox
    }
    #[getter]
    fn r#mod(&self) -> &str {
        &self.inner.r#mod
    }
    #[getter]
    #[pyo3(name = "gen")]
    fn get_gen(&self) -> &str {
        &self.inner.r#gen
    }
    #[getter]
    fn num(&self) -> &str {
        &self.inner.num
    }
    #[getter]
    fn stt(&self) -> &str {
        &self.inner.stt
    }
    #[getter]
    fn cas(&self) -> &str {
        &self.inner.cas
    }
    #[getter]
    fn rat(&self) -> &str {
        &self.inner.rat
    }
    #[getter]
    fn form_gen(&self) -> &str {
        &self.inner.form_gen
    }
    #[getter]
    fn form_num(&self) -> &str {
        &self.inner.form_num
    }
    #[getter]
    fn prc3(&self) -> &str {
        &self.inner.prc3
    }
    #[getter]
    fn prc2(&self) -> &str {
        &self.inner.prc2
    }
    #[getter]
    fn prc1(&self) -> &str {
        &self.inner.prc1
    }
    #[getter]
    fn prc0(&self) -> &str {
        &self.inner.prc0
    }
    #[getter]
    fn enc0(&self) -> &str {
        &self.inner.enc0
    }
    #[getter]
    fn enc1(&self) -> &str {
        &self.inner.enc1
    }
    #[getter]
    fn enc2(&self) -> &str {
        &self.inner.enc2
    }
    #[getter]
    fn d1seg(&self) -> &str {
        &self.inner.d1seg
    }
    #[getter]
    fn d2seg(&self) -> &str {
        &self.inner.d2seg
    }
    #[getter]
    fn d3seg(&self) -> &str {
        &self.inner.d3seg
    }
    #[getter]
    fn atbseg(&self) -> &str {
        &self.inner.atbseg
    }
    #[getter]
    fn d1tok(&self) -> &str {
        &self.inner.d1tok
    }
    #[getter]
    fn d2tok(&self) -> &str {
        &self.inner.d2tok
    }
    #[getter]
    fn d3tok(&self) -> &str {
        &self.inner.d3tok
    }
    #[getter]
    fn atbtok(&self) -> &str {
        &self.inner.atbtok
    }
    #[getter]
    fn bwtok(&self) -> &str {
        &self.inner.bwtok
    }
    #[getter]
    fn root(&self) -> &str {
        &self.inner.root
    }
    #[getter]
    fn pattern(&self) -> &str {
        &self.inner.pattern
    }
    #[getter]
    fn caphi(&self) -> &str {
        &self.inner.caphi
    }
    #[getter]
    fn pos_logprob(&self) -> f64 {
        self.inner.pos_logprob
    }
    #[getter]
    fn lex_logprob(&self) -> f64 {
        self.inner.lex_logprob
    }
    #[getter]
    fn pos_lex_logprob(&self) -> f64 {
        self.inner.pos_lex_logprob
    }

    fn __repr__(&self) -> String {
        format!(
            "ScoredAnalysis(score={}, diac='{}', lex='{}', pos='{}')",
            self.inner.score, self.inner.diac, self.inner.lex, self.inner.pos
        )
    }

    fn __getitem__(&self, key: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let val = self._get_field(key);
            match val {
                FieldValue::Str(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
                FieldValue::F64(f) => Ok(f.into_pyobject(py)?.into_any().unbind()),
                FieldValue::None => Err(pyo3::exceptions::PyKeyError::new_err(key.to_string())),
            }
        })
    }

    #[pyo3(signature = (key, default=None))]
    fn get(&self, py: Python, key: &str, default: Option<PyObject>) -> PyObject {
        let val = self._get_field(key);
        match val {
            FieldValue::Str(s) => s.into_pyobject(py).unwrap().into_any().unbind(),
            FieldValue::F64(f) => f.into_pyobject(py).unwrap().into_any().unbind(),
            FieldValue::None => default.unwrap_or_else(|| py.None()),
        }
    }

    fn __contains__(&self, key: &str) -> bool {
        !matches!(self._get_field(key), FieldValue::None)
    }

    fn keys(&self) -> Vec<&str> {
        vec![
            "score",
            "category",
            "source",
            "diac",
            "lex",
            "bw",
            "gloss",
            "stem",
            "stemcat",
            "stemgloss",
            "catib6",
            "ud",
            "per",
            "asp",
            "vox",
            "mod",
            "gen",
            "num",
            "stt",
            "cas",
            "rat",
            "form_gen",
            "form_num",
            "pos",
            "prc3",
            "prc2",
            "prc1",
            "prc0",
            "enc0",
            "enc1",
            "enc2",
            "d1seg",
            "d2seg",
            "d3seg",
            "atbseg",
            "d1tok",
            "d2tok",
            "d3tok",
            "atbtok",
            "bwtok",
            "root",
            "pattern",
            "caphi",
            "pos_logprob",
            "lex_logprob",
            "pos_lex_logprob",
        ]
    }
}

enum FieldValue<'a> {
    Str(&'a str),
    F64(f64),
    None,
}

impl PyScoredAnalysis {
    fn _get_field(&self, key: &str) -> FieldValue {
        match key {
            "score" => FieldValue::F64(self.inner.score),
            "category" => FieldValue::Str(&self.inner.category),
            "source" => FieldValue::Str(&self.inner.source),
            "diac" => FieldValue::Str(&self.inner.diac),
            "lex" => FieldValue::Str(&self.inner.lex),
            "bw" => FieldValue::Str(&self.inner.bw),
            "gloss" => FieldValue::Str(&self.inner.gloss),
            "stem" => FieldValue::Str(&self.inner.stem),
            "stemcat" => FieldValue::Str(&self.inner.stemcat),
            "stemgloss" => FieldValue::Str(&self.inner.stemgloss),
            "catib6" => FieldValue::Str(&self.inner.catib6),
            "ud" => FieldValue::Str(&self.inner.ud),
            "per" => FieldValue::Str(&self.inner.per),
            "asp" => FieldValue::Str(&self.inner.asp),
            "vox" => FieldValue::Str(&self.inner.vox),
            "mod" => FieldValue::Str(&self.inner.r#mod),
            "gen" => FieldValue::Str(&self.inner.r#gen),
            "num" => FieldValue::Str(&self.inner.num),
            "stt" => FieldValue::Str(&self.inner.stt),
            "cas" => FieldValue::Str(&self.inner.cas),
            "rat" => FieldValue::Str(&self.inner.rat),
            "form_gen" => FieldValue::Str(&self.inner.form_gen),
            "form_num" => FieldValue::Str(&self.inner.form_num),
            "pos" => FieldValue::Str(&self.inner.pos),
            "prc3" => FieldValue::Str(&self.inner.prc3),
            "prc2" => FieldValue::Str(&self.inner.prc2),
            "prc1" => FieldValue::Str(&self.inner.prc1),
            "prc0" => FieldValue::Str(&self.inner.prc0),
            "enc0" => FieldValue::Str(&self.inner.enc0),
            "enc1" => FieldValue::Str(&self.inner.enc1),
            "enc2" => FieldValue::Str(&self.inner.enc2),
            "d1seg" => FieldValue::Str(&self.inner.d1seg),
            "d2seg" => FieldValue::Str(&self.inner.d2seg),
            "d3seg" => FieldValue::Str(&self.inner.d3seg),
            "atbseg" => FieldValue::Str(&self.inner.atbseg),
            "d1tok" => FieldValue::Str(&self.inner.d1tok),
            "d2tok" => FieldValue::Str(&self.inner.d2tok),
            "d3tok" => FieldValue::Str(&self.inner.d3tok),
            "atbtok" => FieldValue::Str(&self.inner.atbtok),
            "bwtok" => FieldValue::Str(&self.inner.bwtok),
            "root" => FieldValue::Str(&self.inner.root),
            "pattern" => FieldValue::Str(&self.inner.pattern),
            "caphi" => FieldValue::Str(&self.inner.caphi),
            "pos_logprob" => FieldValue::F64(self.inner.pos_logprob),
            "lex_logprob" => FieldValue::F64(self.inner.lex_logprob),
            "pos_lex_logprob" => FieldValue::F64(self.inner.pos_lex_logprob),
            _ => FieldValue::None,
        }
    }
}

impl From<ScoredAnalysis> for PyScoredAnalysis {
    fn from(sa: ScoredAnalysis) -> Self {
        PyScoredAnalysis { inner: sa }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DisambiguatedWord {
    #[pyo3(get)]
    word: String,
    #[pyo3(get)]
    analyses: Vec<PyScoredAnalysis>,
}

#[pymethods]
impl DisambiguatedWord {
    fn __getitem__(&self, idx: isize) -> PyResult<PyObject> {
        Python::with_gil(|py| match idx {
            0 | -2 => Ok(self.word.clone().into_pyobject(py)?.into_any().unbind()),
            1 | -1 => Ok(self.analyses.clone().into_pyobject(py)?.into_any().unbind()),
            _ => Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            )),
        })
    }

    fn __len__(&self) -> usize {
        2
    }

    fn __repr__(&self) -> String {
        format!(
            "DisambiguatedWord(word='{}', analyses=[{}])",
            self.word,
            self.analyses
                .iter()
                .map(|a| a.__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pyfunction]
#[pyo3(signature = (sentence, db, model, backoff="NOAN_PROP", top=1))]
fn disambiguate(
    sentence: Vec<String>,
    db: &PyMorphologyDB,
    model: &PyMLEModel,
    backoff: &str,
    top: usize,
) -> PyResult<Vec<DisambiguatedWord>> {
    let refs: Vec<&str> = sentence.iter().map(|s| s.as_str()).collect();
    let results = mle::disambiguate(&refs, &db.inner, &model.inner, backoff, top)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    Ok(refs
        .iter()
        .zip(results.into_iter())
        .map(|(word, word_analyses)| DisambiguatedWord {
            word: word.to_string(),
            analyses: word_analyses
                .into_iter()
                .map(PyScoredAnalysis::from)
                .collect(),
        })
        .collect())
}

#[pyfunction]
#[pyo3(signature = (sentence, mode="compact"))]
fn tokenize(sentence: &str, mode: &str) -> Vec<String> {
    utils::simple_word_tokenize(sentence, mode)
}

#[pyfunction]
#[pyo3(signature = (text, db, model, sep="[+]", scheme="d3tok", preserve_diacritics=false, backoff="NOAN_PROP", fallback=None))]
fn stem(
    text: &str,
    db: &PyMorphologyDB,
    model: &PyMLEModel,
    sep: &str,
    scheme: &str,
    preserve_diacritics: bool,
    backoff: &str,
    fallback: Option<Vec<String>>,
) -> PyResult<String> {
    let fallback_refs: Vec<&str> = fallback
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();
    stemmer::stem(
        text,
        &db.inner,
        &model.inner,
        sep,
        scheme,
        preserve_diacritics,
        backoff,
        None,
        0,
        &fallback_refs,
    )
    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn dediac_ar(text: &str) -> PyResult<String> {
    utils::dediac_ar(text).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

fn resolve_resource_path(path_or_name: &str, component_path: &[&str]) -> PyResult<PathBuf> {
    let direct_path = PathBuf::from(path_or_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    let camel_dir = downloader::get_or_create_camel_dir()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let path = camel_dir.join(path_or_name);
    if !path.exists() {
        let catalogue = downloader::CamelCatalogue::load()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        return catalogue
            .ensure_component_dataset(component_path, Some(path_or_name))
            .map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to download '{}': {}",
                    path_or_name, e
                ))
            });
    }
    Ok(path)
}

#[pyclass]
pub struct MLEDisambiguator {
    db: MorphologyDB,
    model: HashMap<String, ScoredAnalysis>,
}

#[pymethods]
impl MLEDisambiguator {
    #[new]
    #[pyo3(signature = (name="calima-msa-r13"))]
    fn new(name: &str) -> PyResult<Self> {
        let db_path = resolve_resource_path(name, &["MorphologyDB"])?;
        let mle_path = resolve_resource_path(name, &["DisambigMLE"])?;
        let db = MorphologyDB::load(db_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let model = mle::load_mle_model(mle_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(MLEDisambiguator { db, model })
    }

    #[pyo3(signature = (sentence, backoff="NOAN_PROP", top=1))]
    fn disambiguate(
        &self,
        sentence: Vec<String>,
        backoff: &str,
        top: usize,
    ) -> PyResult<Vec<DisambiguatedWord>> {
        let refs: Vec<&str> = sentence.iter().map(|s| s.as_str()).collect();
        let results = mle::disambiguate(&refs, &self.db, &self.model, backoff, top)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(refs
            .iter()
            .zip(results.into_iter())
            .map(|(word, word_analyses)| DisambiguatedWord {
                word: word.to_string(),
                analyses: word_analyses
                    .into_iter()
                    .map(PyScoredAnalysis::from)
                    .collect(),
            })
            .collect())
    }
}

#[pyclass]
pub struct Stemmer {
    db: MorphologyDB,
    model: HashMap<String, ScoredAnalysis>,
    cache: Option<HashMap<String, Vec<ScoredAnalysis>>>,
    max_cache_size: usize,
}

#[pymethods]
impl Stemmer {
    #[new]
    #[pyo3(signature = (name="calima-msa-r13", cache_size=100000))]
    fn new(name: &str, cache_size: usize) -> PyResult<Self> {
        let db_path = resolve_resource_path(name, &["MorphologyDB"])?;
        let mle_path = resolve_resource_path(name, &["DisambigMLE"])?;
        let db = MorphologyDB::load(db_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let model = mle::load_mle_model(mle_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Stemmer {
            db,
            model,
            cache: if cache_size > 0 {
                Some(HashMap::new())
            } else {
                None
            },
            max_cache_size: cache_size,
        })
    }

    #[pyo3(signature = (text, sep="[+]", scheme="d3tok", preserve_diacritics=false, backoff="NOAN_PROP", fallback=None))]
    fn stem(
        &mut self,
        text: &str,
        sep: &str,
        scheme: &str,
        preserve_diacritics: bool,
        backoff: &str,
        fallback: Option<Vec<String>>,
    ) -> PyResult<String> {
        let fallback_refs: Vec<&str> = fallback
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();
        stemmer::stem(
            text,
            &self.db,
            &self.model,
            sep,
            scheme,
            preserve_diacritics,
            backoff,
            self.cache.as_mut(),
            self.max_cache_size,
            &fallback_refs,
        )
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn clear_cache(&mut self) {
        if let Some(c) = &mut self.cache {
            c.clear();
        }
    }

    #[getter]
    fn cache_size(&self) -> usize {
        self.cache.as_ref().map_or(0, |c| c.len())
    }
}

#[pyclass]
pub struct Analyzer {
    db: MorphologyDB,
    backoff: String,
    strict_digit: bool,
}

#[pymethods]
impl Analyzer {
    #[new]
    #[pyo3(signature = (dbname="calima-msa-r13", backoff="NOAN_PROP", strict_digit=false))]
    fn new(dbname: &str, backoff: &str, strict_digit: bool) -> PyResult<Self> {
        let db_path = resolve_resource_path(dbname, &["MorphologyDB"])?;
        let db = MorphologyDB::load(db_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Analyzer {
            db,
            backoff: backoff.to_string(),
            strict_digit,
        })
    }

    #[pyo3(signature = (word))]
    fn analyze(&self, word: &str) -> PyResult<Vec<PyScoredAnalysis>> {
        let results = analyzer::analyze(word, &self.db, self.strict_digit, &self.backoff)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(results.into_iter().map(PyScoredAnalysis::from).collect())
    }
}

#[pymodule]
fn fast_disambig(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMorphologyDB>()?;
    m.add_class::<PyMLEModel>()?;
    m.add_class::<PyScoredAnalysis>()?;
    m.add_class::<DisambiguatedWord>()?;
    m.add_class::<MLEDisambiguator>()?;
    m.add_class::<Stemmer>()?;
    m.add_class::<Analyzer>()?;
    m.add_function(wrap_pyfunction!(disambiguate, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(stem, m)?)?;
    m.add_function(wrap_pyfunction!(dediac_ar, m)?)?;
    Ok(())
}
