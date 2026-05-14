# Translation Linking Feature Analysis & Implementation Plan

**Date:** 2026-05-13  
**Project:** lensisku  
**Feature:** Translation button for linking definitions (Tatoeba-style)

---

## Executive Summary

This document analyzes the current translation linking feature in lensisku and proposes solutions to bridge the architectural gap between Tatoeba's definition-to-definition model and lensisku's corpus-entry-based (valsi) architecture.

**Current State:**
- ✅ "Translate" button exists in DefinitionCard.vue
- ✅ Backend supports bidirectional definition linking (definition_links table)
- ✅ "Link existing" modal allows searching and linking to existing definitions
- ⚠️ Translation workflow requires creating a new corpus entry (valsi) even for phrases
- ⚠️ No direct definition-to-definition creation flow

**Key Challenge:**
Tatoeba links sentences directly (bypassing the concept of "words"), while lensisku requires every definition to belong to a corpus entry (valsi). This creates friction when translating phrases.

---

## Current Architecture Analysis

### 1. Database Schema

```sql
-- Core entities
valsi (corpus entries)
  ├── valsiid (PK)
  ├── word (the actual word/phrase text)
  ├── typeid (word type: gismu, lujvo, cmavo, phrase, etc.)
  └── source_langid (language of the entry itself)

definitions
  ├── definitionid (PK)
  ├── valsiid (FK to valsi) -- REQUIRED
  ├── langid (language of the definition text)
  ├── definition (the actual definition text)
  └── ...

definition_links (bidirectional translation links)
  ├── id (PK)
  ├── definition_id (FK to definitions)
  ├── translation_id (FK to definitions)
  ├── created_by (FK to users)
  └── CONSTRAINT: only allows linking phrases (typeid = 15)
```

**Key Constraint:** Every definition MUST have a parent valsi (corpus entry).

### 2. Current Translation Workflow

When user clicks "Translate" button on a definition:

1. **Frontend (DefinitionCard.vue:line ~390)**
   ```javascript
   router.push(
     `/valsi/add?word=${encodeURIComponent(definition.valsiword)}
      &translate_from_def=${definition.definitionid}`
   )
   ```

2. **UpsertDefinition.vue receives:**
   - `word` query param (pre-filled)
   - `translate_from_def` query param (source definition ID)

3. **Current behavior:**
   - Pre-fills the word field with source valsi word
   - User must fill in definition in their language
   - On submit: creates/finds valsi → creates definition → auto-links via `translate_from_def`

4. **Auto-linking logic (UpsertDefinition.vue:line ~850)**
   ```javascript
   const sourceDefId = queryStr(route.query.translate_from_def)
   if (sourceDefId && !isEditMode.value) {
     await linkDefinitions(sourceDefId, response.data.definition_id)
   }
   ```

### 3. Backend Linking Logic

**File:** `src/jbovlaste/service.rs`

```rust
pub async fn link_definitions(
    pool: &Pool,
    definition_id: i32,
    translation_id: i32,
    user_id: i32,
) -> Result<i32, Box<dyn std::error::Error>> {
    // Validates both definitions are phrases (typeid = 15)
    // Creates bidirectional links in definition_links table
    // Returns link_id
}
```

**Constraints enforced:**
- Both definitions must exist
- Both must be of type "phrase" (typeid = 15)
- Cannot link definition to itself
- Bidirectional: creates both (A→B) and (B→A)

### 4. "Link Existing" Modal

**File:** `frontend/src/components/DefinitionCard.vue`

- Searches for existing phrase definitions
- Allows user to select and link
- Uses same `linkDefinitions()` API
- **Limitation:** Can only link to definitions that already exist

---

## The Tatoeba Model vs. lensisku Model

### Tatoeba Architecture
```
Sentence A (English) ←→ Sentence B (Lojban) ←→ Sentence C (French)
     ↓                        ↓                      ↓
  Direct links          Direct links           Direct links
```

- Sentences are first-class entities
- No intermediate "word" concept
- Direct many-to-many linking
- User adds sentence → immediately linkable

### lensisku Architecture
```
Valsi A (English phrase) → Definition A1 (English)
                                    ↓
                              definition_links
                                    ↓
Valsi B (Lojban phrase)  → Definition B1 (Lojban)
```

- Valsi (corpus entry) is required
- Definition belongs to valsi
- Links are between definitions, not valsi
- User must create valsi before definition can be linked

### The Gap

**Problem:** When translating a phrase:
1. User sees phrase "hello world" in English
2. Wants to add Lojban translation "coi prenu"
3. Must create a NEW valsi entry for "coi prenu" (even though it's just a phrase)
4. This creates corpus pollution with duplicate phrase entries

**Tatoeba's advantage:** No intermediate entity needed. Sentence → Link → Sentence.

---

## Proposed Solutions

### Option A: Streamlined Phrase Creation (Recommended)

**Concept:** Optimize the current workflow to make phrase valsi creation feel invisible.

**Implementation:**

1. **Auto-detect phrase type**
   - When `translate_from_def` is present, automatically set word type to "phrase"
   - Skip word analysis step for phrases
   - Pre-fill source language based on user's preferred language

2. **Simplified UI for translation mode**
   ```
   ┌─────────────────────────────────────────┐
   │ Translating: "hello world" (English)    │
   ├─────────────────────────────────────────┤
   │ Your translation:                       │
   │ [coi prenu                            ] │
   │                                         │
   │ Language: [Lojban ▼]                   │
   │                                         │
   │ Definition:                             │
   │ [A greeting to a person              ] │
   │                                         │
   │ [Create Translation]                    │
   └─────────────────────────────────────────┘
   ```

3. **Backend changes:**
   - Add `POST /jbovlaste/definitions/translate` endpoint
   - Combines valsi creation + definition creation + linking in one transaction
   - Returns the new definition with link confirmation

4. **Database transaction:**
   ```sql
   BEGIN;
     -- Find or create valsi for the phrase
     INSERT INTO valsi (word, typeid, source_langid)
     VALUES ($phrase, 15, $lang)
     ON CONFLICT (word, source_langid) DO UPDATE SET word = EXCLUDED.word
     RETURNING valsiid;
     
     -- Create definition
     INSERT INTO definitions (valsiid, langid, definition, userid)
     VALUES ($valsiid, $lang, $definition, $user)
     RETURNING definitionid;
     
     -- Create bidirectional link
     INSERT INTO definition_links (definition_id, translation_id, created_by)
     VALUES ($source_def, $new_def, $user), ($new_def, $source_def, $user);
   COMMIT;
   ```

**Pros:**
- ✅ Maintains current architecture
- ✅ No schema changes needed
- ✅ Backward compatible
- ✅ Feels like Tatoeba (user doesn't see valsi creation)
- ✅ Reuses existing validation and permissions

**Cons:**
- ⚠️ Still creates valsi entries (but hidden from user)
- ⚠️ Potential for duplicate phrase valsi if not careful

**Effort:** Medium (2-3 days)

---

### Option B: Virtual Valsi for Phrases

**Concept:** Create a special "virtual" valsi that acts as a container for all phrase definitions.

**Implementation:**

1. **Create system valsi:**
   ```sql
   INSERT INTO valsi (word, typeid, source_langid)
   VALUES ('__PHRASE_CONTAINER__', 15, 1);
   ```

2. **Modify definition creation:**
   - When creating phrase definition via translation, use virtual valsi
   - Store actual phrase text in `definition.metadata` JSON field
   - Display phrase text from metadata instead of valsi.word

3. **Update queries:**
   - Modify all definition queries to check for virtual valsi
   - Extract phrase text from metadata when present

**Pros:**
- ✅ No duplicate valsi entries
- ✅ Clean separation of "real words" vs "phrases"
- ✅ Easier to manage phrase translations

**Cons:**
- ❌ Breaks existing architecture assumptions
- ❌ Complex migration for existing phrase definitions
- ❌ Requires changes throughout codebase
- ❌ Metadata field becomes critical (not ideal)

**Effort:** High (1-2 weeks)

---

### Option C: Phrase-Only Definition Table

**Concept:** Create a separate table for phrase definitions that don't require valsi.

**Implementation:**

1. **New table:**
   ```sql
   CREATE TABLE phrase_definitions (
     phrase_definitionid SERIAL PRIMARY KEY,
     phrase_text TEXT NOT NULL,
     langid INTEGER NOT NULL REFERENCES languages(langid),
     definition TEXT NOT NULL,
     userid INTEGER REFERENCES users(userid),
     created_at TIMESTAMPTZ DEFAULT NOW(),
     ...
   );
   ```

2. **Modify definition_links:**
   ```sql
   ALTER TABLE definition_links
   ADD COLUMN definition_type VARCHAR(10) DEFAULT 'valsi',
   ADD COLUMN phrase_definition_id INTEGER REFERENCES phrase_definitions;
   ```

3. **Update all queries:**
   - Union queries between definitions and phrase_definitions
   - Handle both types in linking logic
   - Dual code paths for valsi-based vs phrase-based

**Pros:**
- ✅ Clean separation
- ✅ No valsi pollution
- ✅ Matches Tatoeba model more closely

**Cons:**
- ❌ Major schema change
- ❌ Requires extensive refactoring
- ❌ Duplicate logic for two definition types
- ❌ Complex migration
- ❌ Breaks existing API contracts

**Effort:** Very High (3-4 weeks)

---

### Option D: Hybrid Approach (Quick Win)

**Concept:** Keep current architecture but add UI shortcuts and better UX.

**Implementation:**

1. **Quick translate modal:**
   - Click "Translate" → Opens modal (not new page)
   - Shows source definition in modal header
   - Simple form: phrase text + definition + language
   - Submit → Creates valsi + definition + link in background

2. **Smart valsi reuse:**
   - Before creating new valsi, check if phrase already exists
   - If exists, offer to add definition to existing valsi
   - Show existing definitions for that phrase

3. **Translation suggestions:**
   - When typing phrase, show existing similar phrases
   - Suggest linking to existing definition instead of creating new

**Pros:**
- ✅ Quick to implement
- ✅ Better UX without architecture changes
- ✅ Reduces duplicate valsi creation
- ✅ Can be done incrementally

**Cons:**
- ⚠️ Doesn't fully solve the architectural mismatch
- ⚠️ Still requires valsi creation

**Effort:** Low (3-5 days)

---

## Recommended Implementation Plan

### Phase 1: Quick Wins (Week 1)
**Goal:** Improve current workflow without major changes

1. **Enhance translation page UX**
   - Auto-detect phrase type when `translate_from_def` present
   - Hide word analysis for phrases
   - Simplify form (remove unnecessary fields)
   - Add visual indicator: "Translating from: [source]"

2. **Smart valsi reuse**
   - Check for existing phrase valsi before creating
   - Show warning if duplicate phrase exists
   - Offer to add definition to existing valsi

3. **Better error messages**
   - Clear feedback when linking fails
   - Explain phrase-only restriction
   - Guide user to correct workflow

**Deliverables:**
- Updated UpsertDefinition.vue with translation mode
- Backend validation for duplicate phrases
- User-friendly error messages

---

### Phase 2: Streamlined API (Week 2-3)
**Goal:** Create dedicated translation endpoint

1. **New endpoint: `POST /jbovlaste/definitions/translate`**
   ```typescript
   interface TranslateRequest {
     source_definition_id: number;
     phrase_text: string;
     definition: string;
     lang_id: number;
     notes?: string;
   }
   ```

2. **Atomic transaction**
   - Find or create valsi
   - Create definition
   - Create bidirectional link
   - Return complete result

3. **Frontend integration**
   - New `translateDefinition()` API method
   - Simplified form submission
   - Better loading states

**Deliverables:**
- New Rust endpoint in `src/jbovlaste/controller.rs`
- Service function in `src/jbovlaste/service.rs`
- Frontend API method in `frontend/src/api.ts`
- Updated UpsertDefinition.vue to use new endpoint

---

### Phase 3: Translation Modal (Week 4)
**Goal:** In-place translation without page navigation

1. **New component: TranslationModal.vue**
   - Triggered from DefinitionCard "Translate" button
   - Shows source definition in header
   - Inline form for quick translation
   - Real-time validation

2. **Enhanced search in "Link Existing"**
   - Search across all languages
   - Show existing translations
   - Suggest similar phrases

3. **Translation management**
   - View all translations for a definition
   - Unlink translations
   - Edit linked translations

**Deliverables:**
- TranslationModal.vue component
- Enhanced DefinitionCard.vue
- Translation management UI

---

### Phase 4: Advanced Features (Future)
**Goal:** Tatoeba-like experience

1. **Bulk translation import**
   - Import CSV/TSV of phrase pairs
   - Batch create valsi + definitions + links
   - Progress tracking

2. **Translation suggestions**
   - ML-based translation suggestions
   - Show similar existing translations
   - Community voting on translations

3. **Translation chains**
   - Visualize translation networks
   - A ↔ B ↔ C ↔ D
   - Find indirect translations

---

## Technical Considerations

### 1. Valsi Uniqueness

**Current constraint:**
```sql
UNIQUE(word, source_langid)
```

**Issue:** Multiple users might create same phrase independently.

**Solution:**
- Use `ON CONFLICT DO UPDATE` or `DO NOTHING`
- Add definition to existing valsi instead of failing
- Show existing definitions when phrase exists

### 2. Phrase Type Enforcement

**Current:** Only phrases (typeid = 15) can be linked.

**Consideration:** Should we allow linking other types?
- Lujvo translations?
- Gismu translations?

**Recommendation:** Keep phrase-only for now, expand later if needed.

### 3. Language Pairing

**Current:** No restriction on language pairs.

**Consideration:** Should we enforce language pair rules?
- Lojban ↔ Natural language only?
- Allow Natural ↔ Natural?

**Recommendation:** Allow all pairs, let community decide.

### 4. Permission Model

**Current:** Any logged-in user can create definitions and links.

**Consideration:** Should translation linking require special permission?

**Recommendation:** Keep current model, add moderation tools later.

### 5. Duplicate Detection

**Challenge:** Same phrase in different forms:
- "hello world" vs "Hello World" vs "hello, world"

**Solution:**
- Normalize phrase text (lowercase, trim)
- Show similar phrases during creation
- Allow community to merge duplicates

---

## Migration Strategy

### For Existing Data

1. **Audit existing phrase definitions**
   ```sql
   SELECT v.word, COUNT(d.definitionid) as def_count
   FROM valsi v
   JOIN definitions d ON v.valsiid = d.valsiid
   WHERE v.typeid = 15
   GROUP BY v.word
   HAVING COUNT(d.definitionid) > 1;
   ```

2. **Identify duplicate phrases**
   - Same phrase text, different valsi IDs
   - Merge definitions under single valsi
   - Update definition_links

3. **Clean up orphaned links**
   - Links where one definition was deleted
   - Links between non-phrase definitions

### Backward Compatibility

- All existing endpoints remain functional
- New endpoints are additive
- Frontend gracefully falls back to old workflow
- No breaking changes to API contracts

---

## Success Metrics

### User Experience
- ⏱️ Time to create translation: < 30 seconds (vs current ~2 minutes)
- 📉 Translation abandonment rate: < 10%
- 👍 User satisfaction: > 80% positive feedback

### Data Quality
- 📊 Duplicate phrase valsi: < 5%
- 🔗 Translation link accuracy: > 95%
- ✅ Complete translation chains: > 60%

### System Performance
- ⚡ Translation creation: < 500ms
- 🔍 Search response time: < 200ms
- 💾 Database growth: < 10% increase

---

## Open Questions

1. **Should we allow editing linked translations?**
   - If source changes, notify linked translations?
   - Auto-update vs manual review?

2. **How to handle translation disputes?**
   - Multiple translations for same phrase?
   - Voting system?
   - Moderator review?

3. **Should we support indirect translations?**
   - A → B → C implies A → C?
   - Transitive linking?

4. **How to display translation networks?**
   - Graph visualization?
   - List view?
   - Both?

5. **Should we import Tatoeba data?**
   - Bulk import existing Tatoeba sentences?
   - Attribution and licensing?

---

## Conclusion

**Recommended Approach:** Option A (Streamlined Phrase Creation) + Option D (Hybrid Approach)

This combination provides:
- ✅ Quick wins with minimal risk
- ✅ Maintains architectural integrity
- ✅ Improves UX significantly
- ✅ Incremental implementation
- ✅ Future-proof for advanced features

**Implementation Timeline:**
- Phase 1 (Quick Wins): 1 week
- Phase 2 (Streamlined API): 2 weeks
- Phase 3 (Translation Modal): 1 week
- **Total:** 4 weeks for complete feature

**Next Steps:**
1. Review and approve this plan
2. Create detailed technical specifications
3. Set up feature branch
4. Begin Phase 1 implementation
5. User testing after each phase

---

## Appendix: Code References

### Frontend Files
- `frontend/src/components/DefinitionCard.vue` - Translate button (line ~390)
- `frontend/src/pages/UpsertDefinition.vue` - Translation form (line ~850)
- `frontend/src/api.ts` - API methods for linking

### Backend Files
- `src/jbovlaste/controller.rs` - Link endpoints
- `src/jbovlaste/service.rs` - Link logic (line ~2800)
- `src/jbovlaste/dto.rs` - LinkDefinitionsRequest
- `src/jbovlaste/models.rs` - DefinitionTranslation

### Database
- `migrations/V126__create_definition_links.sql` - Schema
- Table: `definition_links` - Bidirectional links
- Table: `valsi` - Corpus entries
- Table: `definitions` - Definition texts

### Tatoeba Reference
- `archive/tatoeba2/webroot/js/sentences.add_translation.js` - Original implementation
- `archive/tatoeba2/webroot/js/directives/sentence-and-translations.dir.js` - UI logic

---

**Document Version:** 1.0  
**Last Updated:** 2026-05-13  
**Author:** Claude (Kiro AI)  
**Status:** Draft for Review
