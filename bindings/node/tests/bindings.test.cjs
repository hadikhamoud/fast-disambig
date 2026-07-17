const assert = require('node:assert/strict')
const { readFileSync } = require('node:fs')
const { resolve } = require('node:path')
const test = require('node:test')

const { camel, sina } = require('../index.js')
const fixture = JSON.parse(
  readFileSync(resolve(__dirname, '../../../tests/fixtures/camel.json')),
)

test('loads through CommonJS and ESM', async () => {
  const esm = await import('../index.js')
  assert.equal(typeof camel.Stemmer, 'function')
  assert.equal(typeof esm.camel.Stemmer, 'function')
})

test('exports Camel and Sina utilities', () => {
  assert.deepEqual(camel.tokenize(fixture.tokenizeInput, 'full'), fixture.fullTokens)
  assert.equal(camel.dediacAr(fixture.dediacInput), fixture.dediacOutput)
  assert.match(camel.dataDir(), /camel_tools|FAST_DISAMBIG_DATA_DIR/)
  assert.match(sina.assetUrl('catalogue'), /^https:/)
  assert.equal(sina.assetUrl('missing'), null)
})

test(
  'Camel classes provide sync and async parity',
  { timeout: 120_000 },
  async () => {
    const stemmer = await camel.Stemmer.create()
    const offlineStemmer = await camel.Stemmer.create({ allowDownload: false })
    const text = fixture.tokenizeInput
    const expectedStem = fixture.stemOutput

    assert.equal(await stemmer.stem(text), expectedStem)
    assert.equal(await offlineStemmer.stem(text), expectedStem)
    assert.equal(stemmer.stemSync(text), expectedStem)
    assert.equal(stemmer.cacheSize, 2)
    assert.equal(await stemmer.stem(text, { backoff: 'NOAN_ALL' }), expectedStem)
    assert.equal(stemmer.cacheSize, 4)

    const concurrent = await Promise.all([
      stemmer.stem('والكتاب'),
      stemmer.stem('وبالمدرسة'),
      stemmer.stem('فاللغة'),
    ])
    assert.equal(concurrent.length, 3)
    stemmer.clearCache()
    assert.equal(stemmer.cacheSize, 0)

    const disambiguator = await camel.MLEDisambiguator.create()
    const asyncDisambiguated = await disambiguator.disambiguate([
      fixture.disambiguateInput,
    ])
    const syncDisambiguated = disambiguator.disambiguateSync([
      fixture.disambiguateInput,
    ])
    assert.deepEqual(asyncDisambiguated, syncDisambiguated)
    assert.equal(asyncDisambiguated[0].word, fixture.disambiguateInput)
    assert.equal(asyncDisambiguated[0].analyses[0].diac, fixture.topDiac)
    assert.ok(asyncDisambiguated[0].analyses[0].diac)
    assert.ok('category' in asyncDisambiguated[0].analyses[0])
    assert.ok('formGen' in asyncDisambiguated[0].analyses[0])
    assert.ok('d3tok' in asyncDisambiguated[0].analyses[0])

    const analyzer = await camel.Analyzer.create()
    const asyncAnalyses = await analyzer.analyze('والكتاب')
    const syncAnalyses = analyzer.analyzeSync('والكتاب')
    assert.deepEqual(asyncAnalyses, syncAnalyses)
    assert.ok(asyncAnalyses.length > 0)
  },
)

test('async factories reject unknown datasets', { timeout: 30_000 }, async () => {
  await assert.rejects(
    camel.Stemmer.create({
      name: 'definitely-not-a-real-dataset',
      allowDownload: false,
    }),
    /dataset.+not found/i,
  )
})
