# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "fast-disambig",
#     "camel-tools",
#     "datasets",
#     "tqdm",
#     "torch",
#     "transformers",
# ]
# ///

import time
from tqdm import tqdm
from datasets import load_dataset
import fast_disambig
from camel_tools.morphology.analyzer import Analyzer
from camel_tools.morphology.database import MorphologyDB
from camel_tools.disambig.bert import BERTUnfactoredDisambiguator
from camel_tools.tokenizers.word import simple_word_tokenize

NUM_ROWS = 20

print("Loading Hindawi-Books dataset...")
ds = load_dataset("alielfilali01/Hindawi-Books-dataset", split="train")
subset = ds.select(range(NUM_ROWS))
texts = [t for t in subset["ChapterText"] if t]
total_chars = sum(len(t) for t in texts)
print(f"Rows: {len(texts)}, Total chars: {total_chars:,}")
print()


sentences = []
for t in texts:
    tokens = simple_word_tokenize(t)
    if tokens:
        sentences.append(tokens)
total_words = sum(len(s) for s in sentences)
print(f"Sentences: {len(sentences)}, Total words: {total_words:,}")
print()



print("Loading BERT disambiguator with CAMeL analyzer...")
bert_camel = BERTUnfactoredDisambiguator.pretrained(
    "msa", use_gpu=False, pretrained_cache=False, ranking_cache_size=0
)
print("Loaded.")
print()


print("Loading BERT disambiguator with Rust analyzer...")
from camel_tools.data import CATALOGUE
import json
from pathlib import Path
from camel_tools.disambig.score_function import FEATURE_SET_MAP

model_info = CATALOGUE.get_dataset("DisambigBertUnfactored", "msa")
model_config_path = Path(model_info.path, "default_config.json")
with open(model_config_path) as f:
    model_config = json.load(f)

rust_analyzer = fast_disambig.camel.Analyzer("calima-msa-r13")

bert_rust = BERTUnfactoredDisambiguator(
    str(model_info.path),
    rust_analyzer,
    features=FEATURE_SET_MAP[model_config["feature"]],
    scorer=model_config["scorer"],
    tie_breaker=model_config["tie_breaker"],
    use_gpu=False,
    ranking_cache_size=0,
)
print("Loaded.")
print()

print("=" * 60)
print("BERT + CAMeL Analyzer")
print("=" * 60)

start = time.time()
for sent in tqdm(sentences, desc="bert+camel"):
    bert_camel.disambiguate(sent)
elapsed_camel = (time.time() - start) * 1000

print(f"  Sentences: {len(sentences)}")
print(f"  Words:     {total_words}")
print(f"  Time:      {elapsed_camel:,.0f}ms")
print(f"  Words/sec: {total_words / elapsed_camel * 1000:,.0f}")
print()

print("=" * 60)
print("BERT + Rust Analyzer")
print("=" * 60)

start = time.time()
for sent in tqdm(sentences, desc="bert+rust"):
    bert_rust.disambiguate(sent)
elapsed_rust = (time.time() - start) * 1000

print(f"  Sentences: {len(sentences)}")
print(f"  Words:     {total_words}")
print(f"  Time:      {elapsed_rust:,.0f}ms")
print(f"  Words/sec: {total_words / elapsed_rust * 1000:,.0f}")
print()

print("=" * 60)
print(f"Speedup: {elapsed_camel / elapsed_rust:.1f}x")
print("=" * 60)
