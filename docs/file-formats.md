# File formats

Every file or payload lensisku writes for people to keep, move or re-import carries a version. Reading it
runs the same steps: check the format name and version, accept older versions (missing fields take defaults),
refuse newer ones with a clear message, then turn the document into typed values.

| Format | Owner | Current version | Older files | Newer files |
|---|---|---|---|---|
| Full collection export, `format: "lensisku.collection"` | `src/collections/dto.rs` (`COLLECTION_EXPORT_FORMAT_VERSION`), `src/collections/service.rs` (`check_import_format`), `frontend/src/utils/collectionExport.ts` | 2 (`format_version`; missing means 1) | accepted; every version 2 field is optional | refused with 400: "Export format_version N is newer than supported" |

## What a collection export contains

`collection`, `items` and `levels`. Each item carries its content (front and back, language, direction,
`auto_progress`, `script_code`, `is_original`, canonical-form settings, quiz answer, audio as a `sound_url`
data URL) and, for dialogues, a `dialogue` object. The dialogue script in `free_content_front` is the single
source of truth for the lines, per-line glosses and every question (`? fill:`, `? bank:`, `? choice:`). The
parsed `dialogue.structure` in the file is informational and ignored on import.

It never contains user progress: no scores, due dates, review history or per-question ratings.

## Changing a format

1. Bump the version constant on both sides (backend and frontend).
2. Make new fields optional so older files still load, and say what the default is.
3. Add a test that imports a literal sample of the previous version and keep it forever.
   (Open: there is no literal version 1 sample test yet; the round-trip tests in
   `src/collections/import_tests.rs` cover version 2.)
4. Update this page in the same commit.
