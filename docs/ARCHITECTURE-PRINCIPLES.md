# lensisku backend: architecture principles and guidelines

These are the rules the lensisku backend follows, why, and where they are (or are not yet) applied.

## 1. Layering

- Each domain module has `controller.rs` → `service.rs` → `dto.rs` / `models.rs`.
  - **Controllers**: extract request data, authorize, call one service function, map the result
    to an `HttpResponse`. No SQL.
  - **Services**: business logic and SQL. Never build an `HttpResponse`.
  - **DTOs**: parse and validate at the HTTP edge, so services take typed values
    ("one owner per interpretation": normalise once, don't re-parse downstream).
- Reuse an already-open connection/transaction instead of taking a second one from the pool;
  nested pool checkouts under load exhaust the pool.
- Not enforced by tooling yet.

## 2. Error handling

- Never panic on untrusted input: `Result` + an error enum + `?`.
  `unwrap`/`expect`/`panic` are clippy warnings, allowed in tests only (`clippy.toml`).
- Use typed error enums and match on **variants**, never on `e.to_string()`.
  - `AppError` / `AppResult` (crate-wide).
  - `FlashcardError` (`src/flashcards/error.rs`): `LevelNotFound`, `NotFound`, `AccessDenied`,
    `BadRequest`, `Other(String)`, `Source(Box<dyn Error>)`. `Display` reproduces the legacy strings
    and `Source` delegates `Display` and `source()`, so response bodies stay byte-identical.
  - `ExportError` (`src/export/service.rs`): `InvalidLanguage`, `AccessDenied`, `BadRequest`,
    `NotFound`, `Other`.
- Status mapping: validation → 400, auth → 403, missing resource → 404, everything else → 500.
- Controllers keep their existing JSON/plain bodies (the frontend shows `response.data.error`);
  `AppError`'s own `ResponseError` body differs, so it is not used blindly in older controllers.
- Logging goes through the `log` crate; no `println!`/`eprintln!` outside tests.

## 3. Performance

- **Never block the async runtime.** CPU-bound or blocking work goes to `spawn_blocking` /
  `tokio::fs`: bcrypt, SMTP sends (a blocking `lettre` send on the current-thread main runtime
  once stalled pooled connections "idle in transaction" and made requests take 25–106 s after boot;
  now 9–30 ms), PDF export file I/O. Subprocesses have timeouts and `kill_on_drop`.
- **Avoid round trips.** Replace per-row SQL in loops with one set-based statement
  (`UNNEST`, `= ANY($n)`) when behaviour can be kept identical. Done for: thread participants and
  block checks, role permissions, comment hashtags, version-restore keyword mappings, level
  prerequisites, add-cards-to-level, level data in `export_collection_full`.
- **Cache with explicit keys and invalidation.** `index.html` is cached in `utils/seo.rs` and
  reloaded when its mtime/size changes; locale catalogs are parsed once (`LazyLock`).
- **Cap input-derived sizes**: actix payload limits, `pagination.rs` caps, clamp every
  user-controlled limit.
- Justify performance changes with before/after measurements.
- Use typed binds that match column types (`$1::int`, `f64` for DOUBLE PRECISION); mismatches
  surface as runtime 500s.

## 4. Testing and quality gates

- Before every commit: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
  CI (`.github/workflows/rust.yml`) runs the same plus a `db-tests` job.
- `[lints]` in `Cargo.toml`: `unsafe_code = "deny"` (not `forbid`: `ndarray::s!` expands with an
  allow) and a clippy deny-list (`await_holding_lock`, `implicit_clone`, `clone_on_copy`, …).
- DB-backed tests are `#[ignore]` and guarded by `src/test_db_guard.rs`: loopback host only and
  the database name must contain `test`.
- **Pin behaviour before refactoring.** Before batching a loop or changing error plumbing, add a
  test that records current behaviour (e.g. `merge_progress_tests.rs`), then refactor against it.
  Verify user-visible changes through the local HTTP API, compared with `main` in a worktree.

## 5. Decisions

### Applied
- Layering, typed errors (flashcards, export), runtime-blocking fixes, set-based SQL listed above,
  SEO/locale caching, lint deny-list, guarded DB tests, quality gates in CI.
- Collection import loops (`import_json`, `import_collection_from_json`, `import_full`) are batched
  (`UNNEST … WITH ORDINALITY`) behind the pinning test `collections/import_tests.rs`. It also caught
  three bugs, now fixed: duplicate definition ids gave 500, free-content duplicates were compared
  unsanitized, and a warning printed `Some("…")`.
- `merge_anonymous_progress` keeps a level completed once it was, and completes it when the merged
  counts reach the thresholds.
- Lingo exercise mix: `utils/challenge_mix.rs` is the single assigner (quiz → word bank → listen when
  the card has audio, else a flip card). Full import applies it to levels whose items carry no
  `direction` and seeds quiz answers; owners re-run it per level/collection with
  `POST /flashcards/{levels/{id}|collections/{id}}/apply-challenge-mix` (e.g. after adding audio).
  A listen card that cannot be heard shows the translation, never the text to be assembled.
- Full export/import carries item audio as `sound_url` (a `data:audio/...;base64` URL), validated
  like uploads; a bad sound becomes an import warning, and imported audio enables the listen slot.
  Local archive courses (`archive/`, untracked) can embed theirs with `embed_audio.py`.
- Changing a card's direction re-activates progress for the sides it now uses. The progress table
  is unique per `(user, card, side)` including archived rows, so the old
  `ON CONFLICT … WHERE NOT archived DO NOTHING` left owners with no due cards after a re-mix.
- Quiz distractors for free-content cards come from the items' own text. The query used
  `SELECT DISTINCT … ORDER BY RANDOM()`, which Postgres rejects, so every quiz had no options.
- `import_full` logs failed level-prerequisite/membership inserts as warnings instead of `let _ =`.
- CI `db-tests` loads `tests/fixtures/lensisku_schema.sql` (schema + `languages`/`valsitypes`) and
  runs the collections import, merge-progress and challenge-mix progress tests.

### Not applied, and why
- **Splitting into many crates / an `xtask` layer checker**: large, conflict-prone churn with little
  runtime benefit; module boundaries give most of the value. Revisit if compile times hurt.
- **Compile-time-checked SQL (SQLx)**: needs a live DB or offline cache in CI; big migration.
- **Non-default rustfmt settings**: would reformat every file and ruin blame.
- **Batching `merge_anonymous_progress`**: its pinned semantics (duplicate ids count twice,
  completion judged per iteration, unknown id rolls everything back) would change; small,
  user-triggered loop.

## 6. Status-code changes (Oct 2026)

| Endpoint | Before | After |
|---|---|---|
| `PUT /auth/roles/{name}`, unknown role | 200 (silently created) | 404 |
| Full JSON export with ≠1 collection id / non-JSON format | 500 | 400 |
| Full JSON export of a private foreign collection / missing collection | 500 | 403 / 404 |
| Export with unknown `source_lang` | 500 | 400 |
| Level create/update/delete on a foreign collection | 500 | 403 |
| Missing level (update/delete/list cards/add cards) | 500 | 404 |
| Remove card not in level | 500 | 404 |
| Unknown collection id (public flashcards list, ownership/read checks) | 500 | 404 |
| `GET .../levels/{id}/cards` on a locked level | 500 | 403 (same as the due endpoint) |

The frontend only special-cases 401 (`frontend/src/api.ts`); pages show `response.data.error`
generically, so none of these needed UI changes.

### Behaviour changes (Oct 2026, audit follow-up)

- `per_page`/`limit` on every list endpoint is clamped to `1..=pagination::MAX_PER_PAGE` (100).
  The frontend never asks for more than 100.
- `list_flashcards` loads each card's quiz options inside a savepoint; a failure leaves that
  card's `quiz_options` empty instead of aborting the whole list.
- CORS still allows any origin but no longer sends `Access-Control-Allow-Credentials`. The SPA
  calls `/api` on its own origin with a bearer header, so it does not rely on cross-origin cookies.
- Canonical forms are stored trimmed (`"a"`, not `"a "`). Answer checking recomputes both sides,
  so older rows with a trailing space still compare correctly.
- Inferred import levels are numbered densely: "Level N" at position N-1, whatever the source
  level indices were.
- SEO HTML templates are read with `tokio::fs`, so the request thread never blocks on disk.

### Review rating policy (Oct 2026)

- Every stored review rating passes through `flashcards::rating_policy::effective_rating`
  (called from `review_flashcard` and `record_review`, so `/review`, `/fillin`, `/check`,
  `/quiz/submit` and related-card auto-progress all share it).
- The rule only subtracts points. Fixed card types (quiz, image quiz, word bank, listen, fill-in,
  self-assessed, info) keep the submitted rating; only `hint_used: true` loses 1 point. A correct
  fixed quiz answer (client 3) is stored as 3.
- `auto` cards lose 1 point when presented as quiz / word bank / listening (`presented_type`,
  sent by the client and validated against the auto ladder); typing, reverse and flip
  presentations keep the rating. A hint loses 1 more. At most 2 points are subtracted and the
  result is never below 1. A missing `presented_type` on an auto card counts as on-screen.
- Card status: fixed cards move new → learning → review → graduated on stored rating >= 3.
  Auto cards move on stored rating >= `PASS_RATING` (2), but reach review / graduated only after
  at least one correct production answer (typing, reverse, reverse word bank). A stored 1 sends
  any card back to learning. Level stats count stored rating >= 2 as correct.
- A passing review (stored >= 2) whose FSRS interval rounds to 0 days is scheduled 10 minutes
  ahead so the card leaves the current session's due queue.
- `hint_used` defaults to false, so older clients keep working. The flag can never raise a rating.

## 7. Open items

1. `import_full` now warns on failed level inserts, but a failed statement still aborts the
   transaction, so the import fails at commit; savepoints would let it continue.
2. `tests/fixtures/lensisku_schema.sql` must be regenerated when migrations change the tables the
   DB tests use.
3. The export ignored test deletes `cached_dictionary_exports` rows; it needs a disposable copy of
   dictionary data before CI can run it.
4. Background jobs (`background/*`) still have per-row loops; low priority.
