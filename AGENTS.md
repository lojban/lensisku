# AGENTS.md: rules for AI agents and contributors

lensisku is a Lojban dictionary, forum and flashcard-course site: a Rust (actix-web, tokio-postgres,
Redis) backend in `src/` and a Vue 3 frontend in `frontend/`. These rules apply to every change, by
people and by agents, without being asked, unless the person explicitly says otherwise for one task.

The detailed backend design rules are in [docs/ARCHITECTURE-PRINCIPLES.md](docs/ARCHITECTURE-PRINCIPLES.md);
mistakes already made here, each with its rule, are in [docs/pitfalls.md](docs/pitfalls.md). Read both
before your first change.

## Golden rules

1. **Never crash on input.** Everything that arrives over HTTP, in an import file or from the mail archive
   is hostile. No `unwrap`/`expect`/`panic!`/indexing on input-derived values in production code (clippy
   warns; tests may). Use `Result` and typed errors, checked arithmetic, bounded recursion, capped sizes.
   Every crash fix gets a regression test.
2. **Never block the async runtime.** Blocking or CPU-heavy work (bcrypt, SMTP sends, file I/O, TeX) goes to
   `spawn_blocking` or `tokio::fs`. Every external program runs with a deadline, closed stdin and
   `kill_on_drop(true)`, and a failure becomes an error with a plain-English message, never a hang.
3. **Decide on typed values, never on text.** Match on error variants and fields, not on
   `e.to_string()` or on English wording.
4. **One owner per interpretation.** Parse and validate once at the HTTP edge (DTOs); services take typed
   values. Reuse an open connection or transaction instead of taking a second one from the pool.
5. **Persisted and exchanged formats are versioned.** Anything we store or export has a version field; older
   files keep loading, newer ones are refused with a clear message, never half-read. See
   [docs/file-formats.md](docs/file-formats.md).
6. **Export is schema and content, never user progress.** Scores, due dates and review history stay with the
   user. Import must restore everything an export wrote.
7. **No secrets, no personal data** in the repo, tests, logs, error bodies or tool output. Real keys live in
   `.env` and the servers' `secrets` file, never in tracked files. Test values are obvious fakes. Never
   print a secrets file or an environment dump; mask whole values when you must look.
8. **Safe Rust only.** `unsafe_code` stays `deny` (not `forbid`, because `ndarray::s!` expands with an allow).
9. **Tests never touch real services.** No network, no real mailbox, no production database. DB tests are
   `#[ignore]` and guarded by `src/test_db_guard.rs` (loopback host, database name contains `test`). Each
   test case gets its own data.
10. **Say what you verified.** Never claim a live run, a deploy or a delivered email you did not observe.

## Working rules

- Read the code you will touch, and the pitfalls, before you edit. Pin current behaviour with a test before
  refactoring.
- One logical change per commit, with the reason in the message. No unrelated refactors or drive-by
  formatting. Check `git branch --show-current` and `git status` before every commit and push.
- Deploy order is: verify locally, commit and push, deploy to prod, and only then apply data or content
  changes on prod.
- Docs change in the same commit as behaviour: `docs/ARCHITECTURE-PRINCIPLES.md` for design decisions,
  `docs/file-formats.md` for a format change, and `docs/pitfalls.md` when you make a mistake worth not
  repeating (what happened, then the rule; keep it short).
- Work you find but leave for later is written down (a line in `ToDo.md`), not kept in your head.

## Before you commit

```sh
cargo +stable fmt --check      # run last, after every edit (the default nightly has no rustfmt)
cargo clippy --all-targets -- -D warnings
cargo test
cd frontend && pnpm check          # typecheck, lint, format, knip and i18n keys; when the frontend changed
scripts/check-secrets.sh       # no keys, tokens or passwords in what you commit (the pre-commit hook runs it)
```

If a command prints something that looks like a secret, stop, tell the person, and do not repeat it.
