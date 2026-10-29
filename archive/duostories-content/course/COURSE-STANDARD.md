# Course standard (lensisku Lojban courses)

Applies to every course built from this folder (`scripts/build_course.py`) and to
"Simple Lojban in phrases with audio" (`../simple-lojban-in-phrases-with-audio by Gleki [928703]/add_level_dialogues.py`, dialogues in `level_dialogues.py`; corrected items are listed in `audio_exclude.json` so their old recordings are not embedded).

## One level per lesson
A lesson is exactly **one level**. There are no dialogue-only levels. Cards in a level, in order:

1. **Info card** (`just_information`), always first. Front: `Lesson N: <English title> — <Lojban title>`.
   Back: `#### New words` (each word in `{braces}` so it links to the dictionary), then `#### Grammar` (a short note).
2. **Phrase cards**: the lesson's isolated phrases (Lojban front, English back). Directions:
   `quiz_direct` for 1–2 words; otherwise alternate `wordbank_reverse` / `wordbank`.
   Audio courses may use `listen_wordbank` when the card has a sound.
3. **Dialogue card** (`dialogue`), always **last**, and only one per level. Front: the dialogue
   script (`src/dialogue/parser.rs` markup: `# title`, `Name(F): line`, `~ translation`, `> narration`,
   optional `---` questions). Back: the English title and transcript. The dialogue should reuse the
   level's phrases.

A story level (an existing duostories story) has the same shape: info card ("Story N: …",
characters), up to 5 short lines of that story as phrase cards, then the story card last.

Levels are chained linearly (`prerequisite_positions = [previous level]`). `min_cards` = min(5, cards − 1),
`min_success_rate` = 0.8. Card fronts must be unique within a collection, because the importer skips
duplicate free-content fronts. A repeated phrase is shown only the first time.

## Collections
- `course/lojban-through-dialogues_collection.json`: main 30-lesson course (Lifri, Mentu, Linto,
  following `new-dialogue-plan.md`), with stories 01–05 as story levels after lessons 7, 11 and 13.
- `course/lojban-stories-*_collection.json`: the 40 batch-2 stories as four themed story courses
  (10 stories each, in difficulty order). See `STORY_COURSES` in `scripts/course_data.py`.

## Names (cmevla)
- Every name is a valid cmevla: it ends in a consonant and is written with pauses, as in `.lifris.`, `.mentus.`, `.lintas.`.
- It must not contain `la`, `doi`, `lai` or `la'i` unless it is preceded by a consonant.
- A foreign name is respelled phonetically (Sveta → `.sfetas.`, Francesca → `.frantceskas.`).
- A gismu is never a name on its own: lifri → `.lifris.`.
- Names are introduced with `la` (`mi'e la .lifris.`) or with a vocative (`coi .lifris.`, `doi .lintas.`).
- An attitudinal is never followed directly by a bare name. Use `doi`: `.u'u doi .lintas.`.

## Words
Use only words from `korpora2/dictionaries/formal-gismu.tsv`, `lujvo.tsv` or `fuhivla.tsv`, with
their place structures and the place types listed in formal-gismu.tsv. Do not use experimental gismu
(e.g. taksi, kanpe, laldo).

## Validation (required before committing)
Every Lojban string must pass the egglog-lojban parser. That covers phrase fronts, dialogue lines, titles,
question prompts and choice answers.

    python3 scripts/validate_dialogues.py     # dialogues.json, course/course.json, course/*_collection.json
    python3 scripts/validate_dialogues.py "../simple-lojban-in-phrases-with-audio by Gleki [928703]/simple-lojban-in-phrases-with-audio [928703]_collection.json"

The required result is **0 failing non-doubtful sentences**. When you are unsure of a fix, do not guess. Record
it as doubtful: in a `doubtful` entry with where / original / candidate / note in the source, or with a
`collection_note` starting `doubtful:` on the card. Some warnings are known type-checker limitations
(see `checker_limitation` in the validator) and are not failures.
