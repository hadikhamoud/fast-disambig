# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "fast-disambig",
#     "camel-tools",
#     "datasets",
#     "tqdm",
# ]
# ///

import time
from tqdm import tqdm
from datasets import load_dataset
import fast_disambig

NUM_ROWS = 500

print("Loading Hindawi-Books dataset...")
ds = load_dataset("alielfilali01/Hindawi-Books-dataset", split="train")
subset = ds.select(range(NUM_ROWS))
texts = [t for t in subset["ChapterText"] if t]
total_chars = sum(len(t) for t in texts)
print(f"Rows: {len(texts)}, Total chars: {total_chars:,}")
print()

# ── fast-disambig (no cache) ──
print("=" * 60)
print("fast-disambig (no cache)")
print("=" * 60)
stemmer = fast_disambig.camel.Stemmer(cache_size=0)

start = time.time()
for t in tqdm(texts, desc="fast-disambig"):
    stemmer.stem(t)
elapsed_rust = (time.time() - start) * 1000

print(f"  Time:      {elapsed_rust:,.0f}ms")
print(f"  Chars/sec: {total_chars / elapsed_rust * 1000:,.0f}")
print(f"  Avg/row:   {elapsed_rust / len(texts):.1f}ms")
print()

# ── CAMeL Tools ──
from camel_tools.disambig.mle import MLEDisambiguator
from camel_tools.tokenizers.word import simple_word_tokenize

print("=" * 60)
print("CAMeL Tools")
print("=" * 60)
mle = MLEDisambiguator.pretrained()

start = time.time()
for t in tqdm(texts, desc="camel_tools"):
    tokens = simple_word_tokenize(t)
    mle.disambiguate(tokens)
elapsed_camel = (time.time() - start) * 1000

print(f"  Time:      {elapsed_camel:,.0f}ms")
print(f"  Chars/sec: {total_chars / elapsed_camel * 1000:,.0f}")
print(f"  Avg/row:   {elapsed_camel / len(texts):.1f}ms")
print()

# ── Summary ──
print("=" * 60)
print(f"Speedup: {elapsed_camel / elapsed_rust:.1f}x")
print("=" * 60)
