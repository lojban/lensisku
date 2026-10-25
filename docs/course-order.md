# Course order

`/collections/:id?view=course-order[#level-<levelId>]` lists every flashcard of a collection in
level order (level position, then card position inside the level), with each level title as a
heading and an "Unassigned" section for cards that belong to no level. Everyone can read it; only
the collection owner can edit.

## Saving: `PUT /api/flashcards/collections/{collection_id}/course-order` (auth, owner only)

```json
{ "levels": [{ "level_id": 1, "flashcard_ids": [10, 12] }], "level_order": [2, 1] }
```

- `levels`: for each listed level, the full new ordered list of its flashcards (replaces that
  level's membership; levels not listed are untouched, so a card may stay in several levels).
  Moving a card to "Unassigned" = sending its source level without it.
- `level_order` (optional): an exact permutation of all of the collection's level ids.
- Runs in one transaction: ownership, level/card ownership (no cross-collection ids), duplicate
  checks happen first; any error rolls everything back.
- Errors: 403 not owner, 404 collection not found, 400 invalid ids / duplicates / bad permutation.
- Preserves user progress (`user_flashcard_progress`, `user_level_progress`) and all other level
  fields (prerequisites, min cards, min success rate).

Headings use the existing level endpoints: create `POST /flashcards/levels/{cid}`, rename/edit
`PUT /flashcards/levels/{id}`, delete `DELETE /flashcards/levels/{id}` (cards become unassigned
via `ON DELETE CASCADE` on `flashcard_level_items`; no card is deleted).

Frontend: `components/levels/` — `LevelHeading`, `LevelEditModal` (shared by the levels graph,
course order and flashcards page), `CourseOrderView`, `LevelsPanel`.
