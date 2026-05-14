# Translation Workflow Comparison: Tatoeba vs lensisku

## Current lensisku Workflow (Complex)

```
User clicks "Translate" on Definition A
         ↓
Navigate to /valsi/add?word=X&translate_from_def=123
         ↓
UpsertDefinition page loads
         ↓
User sees pre-filled word field
         ↓
User must:
  1. Confirm/edit word text
  2. Select source language
  3. Select definition language  
  4. Run word analysis (for non-phrases)
  5. Fill in definition text
  6. Optionally add notes, etymology, etc.
         ↓
Click "Submit"
         ↓
Backend:
  1. Find or create valsi entry
  2. Create definition
  3. Auto-link to source definition (via translate_from_def)
         ↓
Success: Redirect to new definition page
         ↓
User sees their translation
```

**Steps:** 8-10 user actions  
**Time:** ~2 minutes  
**Friction points:**
- Full page navigation
- Complex form with many fields
- Word analysis step (even for phrases)
- Not obvious that linking happens automatically

---

## Tatoeba Workflow (Simple)

```
User clicks "Translate" on Sentence A
         ↓
Translation form appears inline (no navigation)
         ↓
User sees:
  - Source sentence (read-only)
  - Language selector (pre-filled to user's language)
  - Text input for translation
         ↓
User types translation
         ↓
Press Enter or click "Save"
         ↓
Backend:
  1. Create sentence
  2. Create bidirectional link
         ↓
Success: Translation appears immediately below source
         ↓
User sees their translation in context
```

**Steps:** 2-3 user actions  
**Time:** ~15 seconds  
**Advantages:**
- No page navigation
- Minimal form fields
- Immediate visual feedback
- Clear parent-child relationship

---

## Proposed lensisku Workflow (Streamlined)

### Option 1: Modal Approach (Recommended)

```
User clicks "Translate" on Definition A
         ↓
Modal opens (no navigation)
         ↓
Modal shows:
  ┌─────────────────────────────────────────┐
  │ Translating: "hello world" (English)    │
  │ Definition: A common greeting           │
  ├─────────────────────────────────────────┤
  │ Your translation:                       │
  │ [coi prenu                            ] │
  │                                         │
  │ Language: [Lojban ▼] (auto-detected)  │
  │                                         │
  │ Translation text:                       │
  │ [A greeting to a person              ] │
  │                                         │
  │ [Cancel]  [Create Translation]          │
  └─────────────────────────────────────────┘
         ↓
User fills 2 fields (phrase + definition)
         ↓
Click "Create Translation"
         ↓
Backend (single transaction):
  1. Find or create valsi (hidden from user)
  2. Create definition
  3. Create bidirectional link
         ↓
Success: Modal closes, translation appears in list
         ↓
User sees translation immediately
```

**Steps:** 3-4 user actions  
**Time:** ~30 seconds  
**Improvements:**
- No page navigation
- Only essential fields
- Auto-detects language
- Immediate feedback
- Valsi creation is invisible

### Option 2: Dedicated Endpoint

```
POST /api/jbovlaste/definitions/translate

Request:
{
  "source_definition_id": 123,
  "phrase_text": "coi prenu",
  "definition": "A greeting to a person",
  "lang_id": 2,
  "notes": "Optional notes"
}

Response:
{
  "success": true,
  "definition_id": 456,
  "valsi_id": 789,
  "link_id": 101,
  "translation": {
    "definitionid": 456,
    "valsiword": "coi prenu",
    "definition": "A greeting to a person",
    "langid": 2,
    "lang_name": "Lojban"
  }
}
```

**Advantages:**
- Single API call
- Atomic transaction
- Returns complete result
- Handles all edge cases

---

## Architectural Comparison

### Tatoeba Model
```
┌──────────────┐
│  Sentence    │ (First-class entity)
├──────────────┤
│ - id         │
│ - text       │
│ - lang       │
│ - user_id    │
└──────────────┘
       ↕
┌──────────────┐
│ Sentence     │
│ Links        │ (Many-to-many)
├──────────────┤
│ - sentence_a │
│ - sentence_b │
└──────────────┘
```

**Characteristics:**
- Flat structure
- Direct linking
- No intermediate entities
- Simple queries

### lensisku Current Model
```
┌──────────────┐
│    Valsi     │ (Corpus entry - REQUIRED)
├──────────────┤
│ - valsiid    │
│ - word       │
│ - typeid     │
│ - langid     │
└──────────────┘
       ↓ (1:N)
┌──────────────┐
│ Definition   │ (Belongs to valsi)
├──────────────┤
│ - defid      │
│ - valsiid    │◄─── MUST have parent valsi
│ - definition │
│ - langid     │
└──────────────┘
       ↕
┌──────────────┐
│ Definition   │
│ Links        │ (Many-to-many, phrases only)
├──────────────┤
│ - def_id_a   │
│ - def_id_b   │
└──────────────┘
```

**Characteristics:**
- Hierarchical structure
- Indirect linking (through definitions)
- Valsi required for all definitions
- More complex queries

### lensisku Proposed Model (No Schema Change)
```
┌──────────────┐
│    Valsi     │ (Auto-created, hidden from user)
├──────────────┤
│ - valsiid    │
│ - word       │ ◄─── Auto-populated from phrase
│ - typeid=15  │ ◄─── Always "phrase" for translations
│ - langid     │
└──────────────┘
       ↓ (1:N)
┌──────────────┐
│ Definition   │ (User-facing entity)
├──────────────┤
│ - defid      │
│ - valsiid    │ ◄─── Auto-linked to valsi
│ - definition │ ◄─── User provides this
│ - langid     │ ◄─── User selects this
└──────────────┘
       ↕
┌──────────────┐
│ Definition   │
│ Links        │ (User creates via "Translate")
├──────────────┤
│ - def_id_a   │
│ - def_id_b   │
└──────────────┘
```

**Characteristics:**
- Same schema
- Valsi creation automated
- User focuses on definitions
- Feels like Tatoeba

---

## User Journey Comparison

### Scenario: User wants to translate "hello world" from English to Lojban

#### Current lensisku (8 steps)
1. Find "hello world" definition
2. Click "Translate" button
3. Wait for page load
4. See complex form with many fields
5. Confirm word is "hello world"
6. Select languages (source + definition)
7. Type Lojban translation "coi prenu"
8. Type definition text
9. Click Submit
10. Wait for redirect
11. See result on new page

**Pain points:**
- Lost context (new page)
- Too many fields
- Unclear what happens
- Slow feedback

#### Proposed lensisku (4 steps)
1. Find "hello world" definition
2. Click "Translate" button
3. Modal opens with source visible
4. Type "coi prenu" and definition
5. Click "Create Translation"
6. See result immediately

**Improvements:**
- Context preserved (modal)
- Minimal fields
- Clear purpose
- Fast feedback

#### Tatoeba (3 steps)
1. Find "hello world" sentence
2. Click "Translate" button
3. Type translation
4. Press Enter
5. See result immediately

**Why it's faster:**
- No separate definition field
- Language auto-detected
- Inline editing

---

## Edge Cases & Solutions

### Case 1: Duplicate Phrase Valsi

**Problem:** User tries to translate "hello" but valsi for "hello" already exists.

**Current behavior:** Error or creates duplicate

**Proposed solution:**
```sql
INSERT INTO valsi (word, typeid, source_langid)
VALUES ($1, 15, $2)
ON CONFLICT (word, source_langid) 
DO UPDATE SET word = EXCLUDED.word
RETURNING valsiid;
```
- Reuses existing valsi
- Adds new definition to existing valsi
- No duplicates created

### Case 2: Same Phrase, Different Languages

**Problem:** "hello" in English vs "hello" in French (same spelling, different meaning)

**Solution:** `source_langid` distinguishes them
- English "hello" → valsi with source_langid=1
- French "hello" → valsi with source_langid=3
- Separate valsi entries, no conflict

### Case 3: User Wants to Link to Existing Definition

**Current:** "Link existing" button opens modal

**Proposed:** Keep this! It's complementary.
- "Translate" → Create new translation
- "Link existing" → Link to existing translation

Both workflows coexist.

### Case 4: Translation Chain

**Problem:** A→B→C, user wants to translate A→C directly

**Current:** Must manually create A→C link

**Proposed (future):**
- Show indirect translations
- Suggest creating direct link
- One-click to create A→C from A→B→C

### Case 5: Multiple Translations

**Problem:** "hello" can be "coi", "co'o", "doi", etc.

**Solution:** Allow multiple definitions per valsi
- Valsi: "hello" (English)
  - Definition 1: "coi" (Lojban) - informal greeting
  - Definition 2: "co'o" (Lojban) - parting greeting
  - Definition 3: "doi" (Lojban) - vocative greeting

Each can be linked independently.

---

## Implementation Checklist

### Phase 1: Backend API
- [ ] Create `POST /jbovlaste/definitions/translate` endpoint
- [ ] Implement atomic transaction (valsi + definition + link)
- [ ] Add duplicate valsi handling (ON CONFLICT)
- [ ] Add validation (phrase type, language, etc.)
- [ ] Add error handling with clear messages
- [ ] Write unit tests
- [ ] Write integration tests

### Phase 2: Frontend Modal
- [ ] Create `TranslationModal.vue` component
- [ ] Add modal trigger to DefinitionCard
- [ ] Implement form with validation
- [ ] Add language auto-detection
- [ ] Add loading states
- [ ] Add error display
- [ ] Add success feedback
- [ ] Write component tests

### Phase 3: UX Improvements
- [ ] Update "Translate" button tooltip
- [ ] Add keyboard shortcuts (Ctrl+T to translate)
- [ ] Add translation count badge
- [ ] Show translation preview on hover
- [ ] Add "View all translations" link
- [ ] Improve translation list display

### Phase 4: Polish
- [ ] Add animations (modal open/close)
- [ ] Add optimistic updates
- [ ] Add undo functionality
- [ ] Add translation history
- [ ] Add bulk translation import
- [ ] Add translation export

---

## Performance Considerations

### Database Queries

**Current workflow (3 queries):**
1. Find/create valsi: `INSERT ... ON CONFLICT`
2. Create definition: `INSERT INTO definitions`
3. Create link: `INSERT INTO definition_links`

**Proposed workflow (1 transaction):**
```sql
BEGIN;
  -- Query 1: Upsert valsi
  INSERT INTO valsi ... RETURNING valsiid;
  
  -- Query 2: Insert definition
  INSERT INTO definitions ... RETURNING definitionid;
  
  -- Query 3: Insert bidirectional links
  INSERT INTO definition_links VALUES (...), (...);
COMMIT;
```

**Optimization:** Use prepared statements, connection pooling

### Caching Strategy

**Cache keys:**
- `translations:{definition_id}` → List of translations
- `definition:{definition_id}` → Definition details
- `valsi:{word}:{lang}` → Valsi lookup

**Invalidation:**
- On new translation: invalidate source definition cache
- On unlink: invalidate both definitions
- TTL: 5 minutes for translations, 1 hour for definitions

### Frontend Performance

**Optimizations:**
- Lazy load TranslationModal component
- Debounce search input (300ms)
- Virtual scrolling for long translation lists
- Optimistic UI updates
- Request deduplication

---

## Testing Strategy

### Unit Tests

**Backend:**
```rust
#[test]
async fn test_translate_definition_creates_valsi() { }

#[test]
async fn test_translate_definition_reuses_existing_valsi() { }

#[test]
async fn test_translate_definition_creates_bidirectional_link() { }

#[test]
async fn test_translate_definition_validates_phrase_type() { }
```

**Frontend:**
```typescript
describe('TranslationModal', () => {
  it('opens when translate button clicked')
  it('pre-fills source language')
  it('validates required fields')
  it('calls API with correct payload')
  it('shows success message on completion')
  it('handles API errors gracefully')
})
```

### Integration Tests

```typescript
describe('Translation workflow', () => {
  it('creates translation from definition page')
  it('links definitions bidirectionally')
  it('displays translation in list')
  it('allows unlinking translation')
  it('prevents duplicate links')
})
```

### E2E Tests

```typescript
describe('User translates definition', () => {
  it('completes full translation workflow', async () => {
    await page.goto('/valsi/hello')
    await page.click('[data-testid="translate-button"]')
    await page.fill('[data-testid="phrase-input"]', 'coi')
    await page.fill('[data-testid="definition-input"]', 'greeting')
    await page.click('[data-testid="submit-translation"]')
    await expect(page.locator('[data-testid="translation-list"]'))
      .toContainText('coi')
  })
})
```

---

## Rollout Plan

### Week 1: Backend Foundation
- Day 1-2: Implement translate endpoint
- Day 3: Add validation and error handling
- Day 4-5: Write tests and documentation

### Week 2: Frontend Modal
- Day 1-2: Create TranslationModal component
- Day 3: Integrate with DefinitionCard
- Day 4-5: Add validation and error handling

### Week 3: Testing & Polish
- Day 1-2: Integration testing
- Day 3: E2E testing
- Day 4-5: Bug fixes and polish

### Week 4: Beta & Launch
- Day 1-2: Beta testing with select users
- Day 3: Gather feedback and iterate
- Day 4: Production deployment
- Day 5: Monitor and support

---

## Success Criteria

### Must Have (MVP)
- ✅ User can translate definition in < 30 seconds
- ✅ No duplicate valsi created
- ✅ Bidirectional links created automatically
- ✅ Error messages are clear and actionable
- ✅ Works on mobile and desktop

### Should Have (V1.1)
- ✅ Translation suggestions based on existing translations
- ✅ Bulk translation import
- ✅ Translation history and versioning
- ✅ Keyboard shortcuts

### Nice to Have (V2.0)
- ✅ ML-based translation suggestions
- ✅ Translation quality voting
- ✅ Translation network visualization
- ✅ Collaborative translation editing

---

**Document Version:** 1.0  
**Last Updated:** 2026-05-13  
**Author:** Claude (Kiro AI)
