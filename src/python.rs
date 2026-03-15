use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::analyzer::ScoredAnalysis;
use crate::mle;
use crate::morphology_db::MorphologyDB;
use crate::utils;

#[pyclass]
pub struct PyMorphologyDB {
    inner: MorphologyDB,
}

#[pymethods]
impl PyMorphologyDB {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let db = MorphologyDB::load(PathBuf::from(path))
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
    fn new(path: String) -> PyResult<Self> {
        let model = mle::load_mle_model(PathBuf::from(path))
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

#[pymodule]
fn fast_disambig(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMorphologyDB>()?;
    m.add_class::<PyMLEModel>()?;
    m.add_class::<PyScoredAnalysis>()?;
    m.add_class::<DisambiguatedWord>()?;
    m.add_function(wrap_pyfunction!(disambiguate, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    Ok(())
}
