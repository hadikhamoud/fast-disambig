# fast-disambig

Fast Arabic morphological disambiguation and stemming. Rust engine with Python bindings. Almost drop-in replacement for [CAMeL Tools](https://github.com/CAMeL-Lab/camel_tools) MLE disambiguator (support for other disambiguators soon!)

## Benchmark

Tested on the [Hindawi Books dataset](https://huggingface.co/datasets/alielfilali01/Hindawi-Books-dataset), Apple M1:

| Workload | fast-disambig | CAMeL Tools | Speedup |
|---|---|---|---|
| Single text | 38ms | 340ms | **9x** |
| 491 book chapters (7.1M chars) | 19s | 19m 26s | **61x** |

Reproduce on your machine:

```bash
uv run benchmark.py
```

Share your results in an [issue](https://github.com/hadikhamoud/fast-disambig/issues)!

## Install

```bash
pip install fast-disambig
```

or with uv:

```bash
uv pip install fast-disambig
```

**Requires:** CAMeL Tools data files in `~/.camel_tools/data/`. If missing, they are downloaded automatically on first use. (camel_data CLI tool soon!)

### Build from source

```bash
git clone https://github.com/hadikhamoud/fast-disambig
cd fast-disambig
pip install maturin
maturin develop --release
```

## Usage

### Disambiguator

```python
import fast_disambig

dis = fast_disambig.MLEDisambiguator("calima-msa-r13")  

results = dis.disambiguate(["والكتاب", "الجميل"])
```

### Stemmer

```python
stemmer = fast_disambig.Stemmer()  

# light stemming 

stemmer.stem("والكتاب الجميل")
# 'و[+]ال[+]كتاب ال[+]جميل'

stemmer.stem("وَالْكِتَابُ الْجَمِيلُ", preserve_diacritics=True)
# 'وَ[+]الْ[+]كِتَابُ الْ[+]جَمِيلُ'

stemmer.stem("والكتاب الجميل", sep="_")
# 'و_ال_كتاب ال_جميل'

stemmer.stem("والكتاب الجميل", scheme="d3seg")
# 'و[+]ال[+]كتاب ال[+]جميل'
```

Disable cache:
```python
stemmer = fast_disambig.Stemmer(cache_size=0)
```

### Tokenizer

```python
fast_disambig.tokenize("والكتاب الجميل", "full")
# ['والكتاب', ' ', 'الجميل']

fast_disambig.tokenize("Hello عالم 123!", "full")
# ['Hello', ' ', 'عالم', ' ', '123', '!']
```
