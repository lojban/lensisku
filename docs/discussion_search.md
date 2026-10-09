# Discussion search

`GET /waves/search?search=...` searches site comments, jbotcan/freeforums imports,
mail messages, mirrored wiki pages and native wiki pages (valsi type 16). Empty
searches still browse by activity. Searches default to `sort_by=relevance`; users
can explicitly select time, reactions or replies. Private messages are excluded. Imported wiki pages that also have a native
history definition return the mirrored representation once; native-only pages
remain searchable. Redundant history representations are not embedded.

Migration V178 creates a shared lexical index immediately. Database triggers
maintain source text on imports, edits, renames, redirect changes and deletions.
Collection filters join the current thread attachment; source filters apply to
both lexical and vector retrieval. Ordinary dictionary definitions continue to
use their existing search implementation.

## Ranking and passages

- Literal substring matches escape `%`, `_` and backslashes. Exact titles rank
  first. Explicit precision tiers then rank whole-word phrases, all query words
  in any order, literal substrings, unstemmed full-text matches, English stemmed
  matches, and semantic-only matches. Tiers take precedence over fusion scores;
  title matches and full-text proximity/coverage rank results within each tier.
  Apostrophes remain inside Lojban words, so `na` is not a whole-word match for
  `na'e`. Query punctuation is escaped before regex classification. Titles receive
  greater full-text weight than bodies and author names.
- PostgreSQL `simple` full-text search preserves Lojban/non-English words;
  `english` additionally retrieves English inflections. `websearch_to_tsquery`
  supports quoted phrases, OR and exclusions for the full-text branch; the
  substring branch searches the original literal query.
- Reciprocal rank fusion combines lexical, whole-document and best-passage ranks
  with weights 2, 0.7 and 1 respectively, and rank constant 60. A document receives
  at most one vote per branch, regardless of how many paragraphs match. Explicit
  precision tiers take precedence over this score, and document IDs break ties
  deterministically. Explicit time/reaction/reply sorting retains its own order.
- Semantic candidates require cosine similarity of at least 0.4, configurable via
  `WAVE_SEARCH_MIN_SIMILARITY` (0–1). This is a starting threshold, not a calibrated
  probability; evaluate representative real queries before changing it.
- Semantic retrieval takes the nearest 200 documents and 2,000 passages, collapses
  passages to their best document match, and keeps the nearest 200 distinct
  passage documents. HNSW iterative scans refill filtered searches. Lexical
  candidates have no pagination cap, so searches can reach beyond page 100.
- Response `total` counts the union of lexical matches and qualifying semantic
  candidates, not every document above a similarity threshold in the corpus.
  An empty page still returns its actual total.

The worker drains eight pending documents at a time, with inference batches of
20 passages. Passages follow paragraph breaks. Long paragraphs use overlapping
220-WordPiece windows with 32 tokens of overlap; a title contributes at most 32
WordPieces. Including special tokens stays within the model's 256-token limit.
Mail reply quotations and conventional signatures are removed from semantic
input while remaining available to literal/full-text searches.

A whole-document vector is the normalized mean of all passage vectors. This
includes the entire message instead of embedding only its truncated beginning.
Separate passage vectors find narrow topics near the end of a long message.
The API returns matching text in `relevance.excerpt`. Comment and mail excerpts
use their full-page renderers. Wiki previews hydrate the page Markdown (including
local wiki links) and use the same component as the full article, with bounded
preview length and height. `relevance.score` is a fusion score, not a probability.

Text edits clear both vector levels atomically. Inference runs outside database
transactions; generation checks reject vectors computed before a concurrent
edit. Existing lexical search works during backfill or when the model fails or
`DISABLE_EMBEDDINGS=1`. The worker checks for newly pending documents every minute
when idle; restarts resume unfinished work. Backfill storage/inference cost scales
with the number of passages, so allow it to finish after deployment.

## Validation

```sh
cargo check --bin lensisku
cargo test --bin lensisku waves::
python3 tests/waves_search.py
DISABLE_EMBEDDINGS=0 cargo test --bin lensisku live_model_indexing_and_hybrid_hydration -- --ignored --nocapture
DISABLE_EMBEDDINGS=0 cargo test --bin lensisku passages_retain_tail_and_obey_wordpiece_budget -- --ignored --nocapture
cd frontend && pnpm run typecheck
```

The SQL regression test uses `WAVE_SEARCH_TEST_CONTAINER` (default `lenpostgres`)
and rolls back its disposable schema. It exercises the actual migration/ranking
SQL, all document kinds, exact matches, stemming, literal wildcards, filters,
deep pagination, semantic-only candidates, paragraph deduplication, threshold
rejection and edit/delete invalidation. `tests/waves_ranking.sql` additionally
asserts exact-title/whole-phrase/all-words/substring/semantic ordering against
adversarial vectors that give substring candidates stronger fusion scores. It
checks Lojban apostrophes, Unicode, regex punctuation, whitespace, fallback and
pagination. These are reproducible ranking invariants, not a claim of universal
relevance for every natural-language query. The Rust workflow runs this SQL suite
in a separate disposable pgvector service job. The optional model test uses local `.env`
database configuration, creates and removes a disposable schema, and verifies
real inference, all result hydration paths, backfill and late-paragraph retrieval.

Design references: [PostgreSQL text ranking](https://www.postgresql.org/docs/18/textsearch-controls.html),
[reciprocal rank fusion](https://learn.microsoft.com/en-us/azure/search/hybrid-search-ranking),
[Sentence Transformers sequence limits](https://www.sbert.net/examples/sentence_transformer/applications/computing-embeddings/README.html).
