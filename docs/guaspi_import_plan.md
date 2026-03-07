# Guaspi Dictionary Import Plan

## Overview
The goal of this plan is to import the dictionary of the "Gua\spi" language from the provided `dict.tsv` into the Lensisku database. Lensisku's database schema is designed primarily for Lojban but has support for multiple source languages and rich definition metadata, making it highly adaptable for importing Gua\spi while preserving all provided data columns.

## 1. Schema Extensions (Migrations)
The core language configurations already exist in the database (verified via `dump/jbocma.sql` and live system checks).

* **Verified Languages**: 
  - **Gua\spi**: `art-guaspi` (ID: 316)
  - **Chinese**: `zh` (ID: 6)
  - **Latin**: `la` (ID: 40)
*   **Verified Languages**: 
    -   **Gua\spi**: `art-guaspi` (ID: 316)
    -   **Chinese**: `zh` (ID: 6)
    -   **Latin**: `la` (ID: 40)
    -   **Loglan**: `art-loglan` (ID: 58)
    -   **Lojban**: `jbo` (ID: 1)
    -   **English**: `en` (ID: 2)

As these are already present, no primary language migrations are required.

*   **Gua\spi Valsi Types**: The classes will be mapped to existing Lojban types to maintain compatibility with the UI filters:
    -   `t`: **Skip**. These are structural headers and do not represent individual words.
    -   `S`: Map to **cmavo** (ID: 2).
    -   `P`: Map to **cmavo** (ID: 2).
    -   `r`, `q`, `p`: Map to **gismu** (ID: 1).

## 2. Field Mapping Strategy
The `archive/gaspo/dict.tsv` file has 13 primary columns. We will map them logically into Lensisku's core tables (`valsi`, `definitions`, `natlangwords`/`keywordmapping`).

### `valsi` Table (The Dictionary Entries)
* `source_langid`: Set to 316 (Gua\spi).
* `typeid`: 
  - 2 (cmavo) for classes `S` and `P`.
  - 1 (gismu) for classes `r`, `q`, and `p`.

### `definitions` Table
* `valsiid`: The ID of the newly inserted valsi.
* `langid`: English `langid` (since the descriptions are in English).
* `definition`: Derived from the `definition` column.
* `notes`: Derived from the `Comments` column.
* `Xref`: This column will be appended to the `notes` field with references wrapped in curly brackets (e.g., "See also: {fy}, {tla}, {stl}"). This aligns with Lensisku's internal linking syntax.
* `etymology`: Derived from the combination of `Engl`, `Chinese`, `Latin`, and `Loglan` columns.
* `selmaho`: 
  - For class `S`: Set to "S".
  - For class `P`: Set to "P".
  - Otherwise, use the `Thes` code if applicable.
* `jargon`: For classes `r`, `q`, and `p`, store the respective letter (`r`, `q`, or `p`) in this field to retain the predicate sub-type.
* `metadata`: Lensisku definitions support a JSONB `metadata` column. This is the perfect place to retain the purely informational columns that don't neatly fit the normalized schema:
  ```json
  {
    "thes": "0.1.1",
    "rank": "30",
    "phon": "fe",
    "xref": "fy,tla,stl",
    "class": "S",
    "symbol": "=" 
  }
  ```

### Searchability & Glosswords
While `Engl`, `Chinese`, `Latin`, `Loglan`, and `Lojban` are mapped to the etymology and metadata fields for reference, they should **also** be indexed as searchable gloss words.
* Break down each translation by language.
* Insert/retrieve `natlangwords.wordid`.
* Create `keywordmapping` entries associating the `natlangwordid` to the `definitionid` with `place = 0` (general gloss). This enables Lensisku's search engine to find Gua\spi words using their equivalents in Chinese, Latin, Loglan, or English.

## 3. Implementation Steps
1. **Migrations**: Create `V137__guaspi_valsitypes.sql` to insert the Gua\spi structural valsi types if specific categorization is desired beyond the generic Lojban types.
2. **Import Script**: Create a Rust CLI task (e.g. via `clap` inside `lensisku/src/bin/import_guaspi.rs`) or a specialized endpoint in the `import` controller.
3. **Parse Logic**: 
   - Parse the TSV using the Rust `csv` crate, keeping `delimiter = b'\t'`. 
   - Skip the first 2 header rows. 
   - Discard rows where `!Word` is empty (the `t` class category rows are purely structural headers).
   - Use `tokio_postgres` transaction blocks to atomically write the `valsi` entry, followed by the `definition` entry (with the serialized `metadata` schema object), and then iterate and insert any non-empty `Engl`/`Chinese`/`Latin`/`Loglan`/`Lojban` equivalents into the `natlangwords` mapping.
4. **Resync Cache**: The import script should ensure that the `sync_definition_cache_fields()` trigger runs smoothly to populate the `cached_search_text` for immediate, lightning-fast TSV data search.

## 4. Considerations & Edge Cases
* Some `Rank`, `Phon` and `Chinese` fields have specific internal annotations (`=`, `!`, `-`). Storing these raw in `metadata` will safely preserve them for Gua\spi scholars safely without interfering with Lensisku's parsing patterns.
* Lojban definitions use `<place>` tags generally. The TSV uses regex-like structures for arguments (e.g., `$x_1=$`). We can either keep them exactly as formatted in the dictionary or pre-process them into `$x1$` tags to align with the frontend rendering system, which will beautifully format them with MathJax or custom Lojban place highlighters depending on the UI configuration.
