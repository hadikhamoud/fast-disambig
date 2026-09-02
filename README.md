# fast-disambig

Fast Arabic morphological disambiguation and stemming. Rust engine with Python and Node.js bindings. Almost drop-in replacement for the [CAMeL Tools](https://github.com/CAMeL-Lab/camel_tools) MLE disambiguator (support for other disambiguators, like SinaTools, is in progress).

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

## Python

### Install

```bash
pip install fast-disambig
```

or with uv:

```bash
uv pip install fast-disambig
uv add fast-disambig
```

CAMeL Tools data defaults to `~/.camel_tools/data/`. Missing datasets are downloaded automatically on first use. Set `FAST_DISAMBIG_DATA_DIR` or `CAMELTOOLS_DATA` to use another directory.

Download progress appears automatically in interactive terminals and stays silent in CI or piped server logs. Set `FAST_DISAMBIG_PROGRESS=always` to force progress output or `FAST_DISAMBIG_PROGRESS=never` to disable it.

### Build from source

```bash
git clone https://github.com/hadikhamoud/fast-disambig
cd fast-disambig
pip install maturin
maturin develop --release
```

### Usage

### Disambiguator

```python
import fast_disambig

dis = fast_disambig.camel.MLEDisambiguator("calima-msa-r13")  

results = dis.disambiguate(["والكتاب", "الجميل"])
```

### Stemmer

```python
stemmer = fast_disambig.camel.Stemmer()  

# light stemming 
stemmer.stem("والكتاب الجميل")
# 'و[+]ال[+]كتاب ال[+]جميل'

stemmer.stem("وَالْكِتَابُ الْجَمِيلُ", preserve_diacritics=True)
# 'وَ[+]الْ[+]كِتَابُ الْ[+]جَمِيلُ'

stemmer.stem("والكتاب الجميل", sep="_")
# 'و_ال_كتاب ال_جميل'

stemmer.stem("والكتاب الجميل", scheme="d3seg")
# 'و[+]ال[+]كتاب ال[+]جميل'

# fallback: try d3tok first, then d3seg, then bwtok if merge fails
stemmer.stem("والكتاب الجميل", fallback=["d3seg", "bwtok"])
# 'و[+]ال[+]كتاب ال[+]جميل'
```

Disable cache:
```python
stemmer = fast_disambig.camel.Stemmer(cache_size=0)
```

### Tagged stemming

`stem_tagged` performs the same segmentation as `stem` but returns one `StemPiece` per
piece, separator, whitespace or punctuation token. `ud` is the Universal Dependencies tag
of the piece (`SPEC_TOK` for separators, `UNK` for whitespace or unanalysable words) and
`pos` is the CAMeL POS tag of the word the piece came from. Joining `text` over the pieces
gives exactly the output of `stem`.

```python
pieces = stemmer.stem_tagged("والكتاب الجميل")
[(p.text, p.ud, p.pos) for p in pieces]
# [('و', 'CCONJ', 'noun'), ('[+]', 'SPEC_TOK', 'noun'), ('ال', 'NOUN', 'noun'),
#  ('[+]', 'SPEC_TOK', 'noun'), ('كتاب', 'NOUN', 'noun'), (' ', 'UNK', 'UNK'),
#  ('ال', 'NOUN', 'noun'), ('[+]', 'SPEC_TOK', 'noun'), ('جميل', 'NOUN', 'noun')]

"".join(p.text for p in pieces) == stemmer.stem("والكتاب الجميل")
# True
```

### Tokenizer

```python
fast_disambig.camel.tokenize("والكتاب الجميل", "full")
# ['والكتاب', ' ', 'الجميل']

fast_disambig.camel.tokenize("Hello عالم 123!", "full")
# ['Hello', ' ', 'عالم', ' ', '123', '!']
```

## Node.js

The Node addon uses napi-rs and runs model loading and heavy analysis work away from the JavaScript event loop.

### Install

```bash
npm install fast-disambig
```

Build locally from this repository:

```bash
cd bindings/node
npm install
npm run build
npm test
```

### Stemmer

```ts
import { camel, sina } from 'fast-disambig'

const stemmer = await camel.Stemmer.create({
  name: 'calima-msa-r13',
  cacheSize: 100_000,
  allowDownload: true,
})

const stemmed = await stemmer.stem('والكتاب الجميل', {
  scheme: 'd3tok',
  fallback: ['d3seg', 'bwtok'],
})

// Sync variants are intended for scripts, not request handlers.
const syncResult = stemmer.stemSync('والكتاب الجميل')
```

### Disambiguator

```ts
const disambiguator = await camel.MLEDisambiguator.create()
const words = await disambiguator.disambiguate(['والكتاب', 'الجميل'], {
  backoff: 'NOAN_PROP',
  top: 1,
})

console.log(words[0].analyses[0].diac)
```

### Analyzer

```ts
const analyzer = await camel.Analyzer.create({
  backoff: 'NOAN_PROP',
  strictDigit: false,
})

const analyses = await analyzer.analyze('والكتاب')
```

### Utilities And Sina

```ts
camel.tokenize('والكتاب الجميل', 'full')
camel.dediacAr('وَالْكِتَابُ')

// Sina currently exposes its implemented data catalogue/downloader surface.
await sina.listDatasets()
await sina.downloadDataset('morph')
```

Set `FAST_DISAMBIG_SINA_DATA_DIR` to override Sina's default `~/.sinatools` directory.

### Next.js

Use the Node runtime and keep the native package external:

```js
// next.config.js
module.exports = {
  serverExternalPackages: ['fast-disambig'],
}
```

```ts
export const runtime = 'nodejs'
```

Create one model instance, or one creation promise, at module scope rather than loading it per request. Native addons do not run in the Edge runtime. For serverless deployments, package the CAMeL data with the application, set `FAST_DISAMBIG_DATA_DIR`, and use `allowDownload: false` to fail immediately when data is missing.

## Architecture

```text
src/                 Shared Rust core and CLI
bindings/python/     PyO3 adapter built by maturin
bindings/node/       napi-rs adapter and npm package
```

The binding crates only translate language values and errors. Camel/Sina algorithms, model loading, resource resolution, and thread-safe engine state live in the shared core.

## Releases

Publishing a GitHub Release builds Python wheels and six Node prebuilds, then publishes to PyPI and npm. npm publication uses GitHub OIDC trusted publishing and does not use a repository token.

The version in the root `Cargo.toml` under `[workspace.package]` is the single source of truth. CI synchronizes `package.json`, `package-lock.json`, and every platform manifest before building. Release tags must match that version exactly, such as Cargo `0.3.0` with tag `v0.3.0`.

Configure the following trusted publisher on `fast-disambig` and each generated platform package:

```text
Provider: GitHub Actions
Repository: hadikhamoud/fast-disambig
Workflow: release.yml
Environment: npm
Allowed action: npm publish
```

npm requires a package to exist before trusted publishing can be configured. For the first npm release only, reserve the seven new package names with an interactive bootstrap publication, configure the trusted publisher on each package, and then use the normal GitHub Release workflow. Subsequent releases are fully tokenless and receive automatic npm provenance attestations.
