# Translation Feature Implementation Guide

**Date:** 2026-05-13  
**Project:** lensisku  
**Feature:** Streamlined Translation Linking

---

## Quick Start

This guide provides concrete code examples for implementing the streamlined translation feature.

---

## Backend Implementation

### 1. New DTO (Data Transfer Object)

**File:** `src/jbovlaste/dto.rs`

```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TranslateDefinitionRequest {
    /// Source definition ID to translate from
    pub source_definition_id: i32,
    
    /// The phrase text in target language
    #[validate(length(min = 1, max = 500))]
    pub phrase_text: String,
    
    /// Definition text in target language
    #[validate(length(min = 1, max = 10000))]
    pub definition: String,
    
    /// Target language ID
    pub lang_id: i32,
    
    /// Optional notes
    #[validate(length(max = 5000))]
    pub notes: Option<String>,
    
    /// Optional etymology
    pub etymology: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TranslateDefinitionResponse {
    pub success: bool,
    pub definition_id: i32,
    pub valsi_id: i32,
    pub link_id: i32,
    pub translation: DefinitionTranslation,
}
```

### 2. Service Function

**File:** `src/jbovlaste/service.rs`

```rust
pub async fn translate_definition(
    pool: &Pool,
    req: TranslateDefinitionRequest,
    user_id: i32,
) -> Result<TranslateDefinitionResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // 1. Validate source definition exists and is a phrase
    let source_row = transaction
        .query_one(
            "SELECT d.definitionid, v.typeid, v.word, d.langid
             FROM definitions d
             JOIN valsi v ON d.valsiid = v.valsiid
             WHERE d.definitionid = $1",
            &[&req.source_definition_id],
        )
        .await?;

    let source_typeid: i16 = source_row.get("typeid");
    if source_typeid != 15 {
        return Err("Source definition must be a phrase".into());
    }

    // 2. Normalize phrase text
    let normalized_phrase = req.phrase_text.trim().to_lowercase();

    // 3. Find or create valsi for the phrase
    let valsi_row = transaction
        .query_one(
            "INSERT INTO valsi (word, typeid, source_langid)
             VALUES ($1, 15, $2)
             ON CONFLICT (word, source_langid) 
             DO UPDATE SET word = EXCLUDED.word
             RETURNING valsiid",
            &[&normalized_phrase, &req.lang_id],
        )
        .await?;

    let valsi_id: i32 = valsi_row.get("valsiid");

    // 4. Create definition
    let def_row = transaction
        .query_one(
            "INSERT INTO definitions (
                valsiid, langid, definition, notes, etymology, userid, time
             )
             VALUES ($1, $2, $3, $4, $5, $6, EXTRACT(EPOCH FROM NOW())::INTEGER)
             RETURNING definitionid, created_at",
            &[
                &valsi_id,
                &req.lang_id,
                &req.definition,
                &req.notes,
                &req.etymology,
                &user_id,
            ],
        )
        .await?;

    let definition_id: i32 = def_row.get("definitionid");
    let created_at: DateTime<Utc> = def_row.get("created_at");

    // 5. Create bidirectional links
    let link_row = transaction
        .query_one(
            "INSERT INTO definition_links (definition_id, translation_id, created_by)
             VALUES ($1, $2, $3), ($2, $1, $3)
             ON CONFLICT (definition_id, translation_id) DO NOTHING
             RETURNING id",
            &[&req.source_definition_id, &definition_id, &user_id],
        )
        .await?;

    let link_id: i32 = link_row.get("id");

    // 6. Get language name
    let lang_row = transaction
        .query_one(
            "SELECT realname FROM languages WHERE langid = $1",
            &[&req.lang_id],
        )
        .await?;

    let lang_name: String = lang_row.get("realname");

    transaction.commit().await?;

    // 7. Build response
    Ok(TranslateDefinitionResponse {
        success: true,
        definition_id,
        valsi_id,
        link_id,
        translation: DefinitionTranslation {
            definitionid: definition_id,
            valsiword: normalized_phrase.clone(),
            definition: req.definition,
            langid: req.lang_id,
            lang_name,
            link_id,
        },
    })
}
```

### 3. Controller Endpoint

**File:** `src/jbovlaste/controller.rs`

```rust
#[utoipa::path(
    post,
    path = "/jbovlaste/definitions/translate",
    tag = "jbovlaste",
    request_body = TranslateDefinitionRequest,
    responses(
        (status = 200, description = "Translation created successfully", body = TranslateDefinitionResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Create translation",
    description = "Creates a new translation for a phrase definition. Automatically creates or reuses valsi entry and establishes bidirectional link."
)]
#[post("/definitions/translate")]
#[protect("edit_definition")]
pub async fn translate_definition_handler(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<TranslateDefinitionRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": format!("Validation error: {}", e)
        }));
    }

    match service::translate_definition(&pool, req.into_inner(), claims.sub).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("must be a phrase") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": msg
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": msg
                }))
            }
        }
    }
}
```

### 4. Register Route

**File:** `src/jbovlaste/mod.rs`

```rust
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jbovlaste")
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .service(controller::translate_definition_handler) // Add this line
                    .service(controller::link_definitions_handler)
                    // ... other routes
            )
            // ... rest of config
    );
}
```

---

## Frontend Implementation

### 1. API Method

**File:** `frontend/src/api.ts`

```typescript
export interface TranslateDefinitionRequest {
  source_definition_id: number
  phrase_text: string
  definition: string
  lang_id: number
  notes?: string
  etymology?: string
}

export interface TranslateDefinitionResponse {
  success: boolean
  definition_id: number
  valsi_id: number
  link_id: number
  translation: {
    definitionid: number
    valsiword: string
    definition: string
    langid: number
    lang_name: string
    link_id: number
  }
}

export const translateDefinition = (data: TranslateDefinitionRequest) =>
  api.post<TranslateDefinitionResponse>('/jbovlaste/definitions/translate', data)
```

### 2. Translation Modal Component

**File:** `frontend/src/components/TranslationModal.vue`

```vue
<template>
  <Modal :show="show" :title="t('translation.modalTitle')" @close="$emit('close')">
    <div class="space-y-4">
      <!-- Source Definition Display -->
      <div class="bg-gray-50 p-4 rounded-lg border">
        <div class="text-sm text-gray-600 mb-1">
          {{ t('translation.translatingFrom') }}
        </div>
        <div class="font-semibold text-gray-900">{{ sourceDefinition.valsiword }}</div>
        <div class="text-sm text-gray-700 mt-2">{{ sourceDefinition.definition }}</div>
        <div class="text-xs text-gray-500 mt-1">
          {{ sourceDefinition.lang_name }}
        </div>
      </div>

      <!-- Translation Form -->
      <form @submit.prevent="handleSubmit">
        <!-- Language Selector -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            {{ t('translation.targetLanguage') }}
            <span class="text-red-500">*</span>
          </label>
          <select
            v-model="form.lang_id"
            required
            class="input-field w-full"
            :disabled="isSubmitting"
          >
            <option v-for="lang in languages" :key="lang.langid" :value="lang.langid">
              {{ lang.realname }}
            </option>
          </select>
        </div>

        <!-- Phrase Text -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            {{ t('translation.phraseText') }}
            <span class="text-red-500">*</span>
          </label>
          <input
            v-model="form.phrase_text"
            type="text"
            required
            maxlength="500"
            class="input-field w-full"
            :placeholder="t('translation.phraseTextPlaceholder')"
            :disabled="isSubmitting"
          />
          <div class="text-xs text-gray-500 mt-1">
            {{ form.phrase_text.length }} / 500
          </div>
        </div>

        <!-- Definition Text -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            {{ t('translation.definitionText') }}
            <span class="text-red-500">*</span>
          </label>
          <textarea
            v-model="form.definition"
            required
            rows="4"
            maxlength="10000"
            class="input-field w-full"
            :placeholder="t('translation.definitionPlaceholder')"
            :disabled="isSubmitting"
          />
          <div class="text-xs text-gray-500 mt-1">
            {{ form.definition.length }} / 10000
          </div>
        </div>

        <!-- Notes (Optional) -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            {{ t('translation.notes') }}
            <span class="text-gray-400 text-xs">({{ t('common.optional') }})</span>
          </label>
          <textarea
            v-model="form.notes"
            rows="2"
            maxlength="5000"
            class="input-field w-full"
            :placeholder="t('translation.notesPlaceholder')"
            :disabled="isSubmitting"
          />
        </div>

        <!-- Error Display -->
        <AlertComponent v-if="error" type="error" class="mb-4">
          {{ error }}
        </AlertComponent>

        <!-- Actions -->
        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="ui-btn--empty"
            :disabled="isSubmitting"
            @click="$emit('close')"
          >
            {{ t('common.cancel') }}
          </button>
          <Button
            type="submit"
            variant="primary"
            :loading="isSubmitting"
            :disabled="!isFormValid"
          >
            <template #icon>
              <Languages class="h-5 w-5" />
            </template>
            {{ t('translation.createTranslation') }}
          </Button>
        </div>
      </form>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { translateDefinition } from '@/api'
import Modal from './Modal.vue'
import Button from './Button.vue'
import AlertComponent from './AlertComponent.vue'
import { Languages } from 'lucide-vue-next'

interface Props {
  show: boolean
  sourceDefinition: {
    definitionid: number
    valsiword: string
    definition: string
    langid: number
    lang_name: string
  }
  languages: Array<{ langid: number; realname: string }>
  userPreferredLangId?: number
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  success: [translationId: number]
}>()

const { t } = useI18n()

const form = ref({
  lang_id: props.userPreferredLangId || 2, // Default to Lojban
  phrase_text: '',
  definition: '',
  notes: '',
})

const isSubmitting = ref(false)
const error = ref('')

const isFormValid = computed(() => {
  return (
    form.value.phrase_text.trim().length > 0 &&
    form.value.definition.trim().length > 0 &&
    form.value.lang_id > 0
  )
})

// Reset form when modal opens
watch(
  () => props.show,
  (newShow) => {
    if (newShow) {
      form.value = {
        lang_id: props.userPreferredLangId || 2,
        phrase_text: '',
        definition: '',
        notes: '',
      }
      error.value = ''
    }
  }
)

const handleSubmit = async () => {
  if (!isFormValid.value) return

  isSubmitting.value = true
  error.value = ''

  try {
    const response = await translateDefinition({
      source_definition_id: props.sourceDefinition.definitionid,
      phrase_text: form.value.phrase_text.trim(),
      definition: form.value.definition.trim(),
      lang_id: form.value.lang_id,
      notes: form.value.notes.trim() || undefined,
    })

    emit('success', response.data.definition_id)
    emit('close')
  } catch (e: any) {
    console.error('Translation error:', e)
    error.value =
      e.response?.data?.error || t('translation.errorCreating')
  } finally {
    isSubmitting.value = false
  }
}
</script>
```

### 3. Update DefinitionCard

**File:** `frontend/src/components/DefinitionCard.vue`

Add to script section:

```typescript
// Add to imports
import TranslationModal from './TranslationModal.vue'

// Add to reactive state
const showTranslationModal = ref(false)
const languages = ref<Array<{ langid: number; realname: string }>>([])

// Add method to load languages
const loadLanguages = async () => {
  try {
    const res = await getLanguages()
    languages.value = res.data
  } catch (e) {
    console.error('Failed to load languages:', e)
  }
}

// Load languages on mount
onMounted(() => {
  loadLanguages()
})

// Handle translation success
const handleTranslationSuccess = (translationId: number) => {
  showTranslationModal.value = false
  emit('refresh-definitions')
  showSuccess(t('translation.successMessage'))
}
```

Update template:

```vue
<!-- Replace the translate button click handler -->
<button
  v-if="auth.state.isLoggedIn && canTranslate"
  class="ui-btn--link ui-btn--group-item inline-flex items-center justify-center gap-2"
  :title="
    definition.type_name === 'phrase'
      ? t('components.definitionCard.translateButtonTitlePhrase')
      : t('components.definitionCard.translateButtonTitle')
  "
  @click="showTranslationModal = true"
>
  <Languages class="h-4 w-4 shrink-0" />
  {{ t('components.definitionCard.translateButton') }}
</button>

<!-- Add modal at end of template -->
<TranslationModal
  :show="showTranslationModal"
  :source-definition="definition"
  :languages="languages"
  :user-preferred-lang-id="auth.state.user?.preferred_lang_id"
  @close="showTranslationModal = false"
  @success="handleTranslationSuccess"
/>
```

### 4. Translations (i18n)

**File:** `frontend/src/locales/en.json`

```json
{
  "translation": {
    "modalTitle": "Create Translation",
    "translatingFrom": "Translating from:",
    "targetLanguage": "Target Language",
    "phraseText": "Translation Text",
    "phraseTextPlaceholder": "Enter the translated phrase",
    "definitionText": "Definition",
    "definitionPlaceholder": "Explain what this phrase means",
    "notes": "Notes",
    "notesPlaceholder": "Optional notes about this translation",
    "createTranslation": "Create Translation",
    "errorCreating": "Failed to create translation. Please try again.",
    "successMessage": "Translation created successfully!"
  }
}
```

---

## Database Migration

**File:** `migrations/V128__add_translation_indexes.sql`

```sql
-- Add indexes for better translation query performance
CREATE INDEX IF NOT EXISTS idx_definitions_langid_valsiid 
ON definitions(langid, valsiid);

CREATE INDEX IF NOT EXISTS idx_valsi_word_langid 
ON valsi(word, source_langid);

-- Add function to get translation count
CREATE OR REPLACE FUNCTION get_translation_count(def_id INTEGER)
RETURNS INTEGER AS $$
  SELECT COUNT(*)::INTEGER
  FROM definition_links
  WHERE definition_id = def_id;
$$ LANGUAGE SQL STABLE;

-- Add view for translation pairs
CREATE OR REPLACE VIEW translation_pairs AS
SELECT 
  dl.id as link_id,
  d1.definitionid as source_id,
  v1.word as source_word,
  d1.definition as source_definition,
  l1.realname as source_lang,
  d2.definitionid as target_id,
  v2.word as target_word,
  d2.definition as target_definition,
  l2.realname as target_lang,
  dl.created_at
FROM definition_links dl
JOIN definitions d1 ON dl.definition_id = d1.definitionid
JOIN valsi v1 ON d1.valsiid = v1.valsiid
JOIN languages l1 ON d1.langid = l1.langid
JOIN definitions d2 ON dl.translation_id = d2.definitionid
JOIN valsi v2 ON d2.valsiid = v2.valsiid
JOIN languages l2 ON d2.langid = l2.langid
WHERE d1.definitionid < d2.definitionid; -- Avoid duplicates
```

---

## Testing

### Backend Unit Test

**File:** `src/jbovlaste/service_test.rs`

```rust
#[tokio::test]
async fn test_translate_definition() {
    let pool = setup_test_pool().await;
    
    // Create source definition
    let source_id = create_test_phrase_definition(&pool, "hello", 1).await;
    
    // Translate it
    let req = TranslateDefinitionRequest {
        source_definition_id: source_id,
        phrase_text: "coi".to_string(),
        definition: "A greeting".to_string(),
        lang_id: 2,
        notes: None,
        etymology: None,
    };
    
    let result = translate_definition(&pool, req, 1).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.translation.valsiword, "coi");
    assert_eq!(response.translation.langid, 2);
    
    // Verify bidirectional link
    let links = get_definition_translations(&pool, source_id).await.unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].definitionid, response.definition_id);
}

#[tokio::test]
async fn test_translate_reuses_existing_valsi() {
    let pool = setup_test_pool().await;
    
    // Create valsi for "coi"
    let valsi_id = create_test_valsi(&pool, "coi", 2).await;
    
    // Create source definition
    let source_id = create_test_phrase_definition(&pool, "hello", 1).await;
    
    // Translate to "coi" (should reuse existing valsi)
    let req = TranslateDefinitionRequest {
        source_definition_id: source_id,
        phrase_text: "coi".to_string(),
        definition: "A greeting".to_string(),
        lang_id: 2,
        notes: None,
        etymology: None,
    };
    
    let result = translate_definition(&pool, req, 1).await.unwrap();
    assert_eq!(result.valsi_id, valsi_id);
}
```

### Frontend Component Test

**File:** `frontend/src/components/__tests__/TranslationModal.spec.ts`

```typescript
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TranslationModal from '../TranslationModal.vue'
import * as api from '@/api'

vi.mock('@/api')

describe('TranslationModal', () => {
  const mockSourceDefinition = {
    definitionid: 1,
    valsiword: 'hello',
    definition: 'A greeting',
    langid: 1,
    lang_name: 'English',
  }

  const mockLanguages = [
    { langid: 1, realname: 'English' },
    { langid: 2, realname: 'Lojban' },
  ]

  it('renders when show is true', () => {
    const wrapper = mount(TranslationModal, {
      props: {
        show: true,
        sourceDefinition: mockSourceDefinition,
        languages: mockLanguages,
      },
    })

    expect(wrapper.text()).toContain('hello')
    expect(wrapper.text()).toContain('A greeting')
  })

  it('validates required fields', async () => {
    const wrapper = mount(TranslationModal, {
      props: {
        show: true,
        sourceDefinition: mockSourceDefinition,
        languages: mockLanguages,
      },
    })

    const submitButton = wrapper.find('button[type="submit"]')
    expect(submitButton.attributes('disabled')).toBeDefined()

    await wrapper.find('input[type="text"]').setValue('coi')
    await wrapper.find('textarea').setValue('A greeting in Lojban')

    expect(submitButton.attributes('disabled')).toBeUndefined()
  })

  it('calls API on submit', async () => {
    const mockTranslate = vi.mocked(api.translateDefinition)
    mockTranslate.mockResolvedValue({
      data: {
        success: true,
        definition_id: 2,
        valsi_id: 3,
        link_id: 4,
        translation: {
          definitionid: 2,
          valsiword: 'coi',
          definition: 'A greeting in Lojban',
          langid: 2,
          lang_name: 'Lojban',
          link_id: 4,
        },
      },
    })

    const wrapper = mount(TranslationModal, {
      props: {
        show: true,
        sourceDefinition: mockSourceDefinition,
        languages: mockLanguages,
      },
    })

    await wrapper.find('input[type="text"]').setValue('coi')
    await wrapper.find('textarea').setValue('A greeting in Lojban')
    await wrapper.find('form').trigger('submit')

    expect(mockTranslate).toHaveBeenCalledWith({
      source_definition_id: 1,
      phrase_text: 'coi',
      definition: 'A greeting in Lojban',
      lang_id: 2,
      notes: undefined,
    })
  })

  it('emits success event on successful translation', async () => {
    const mockTranslate = vi.mocked(api.translateDefinition)
    mockTranslate.mockResolvedValue({
      data: {
        success: true,
        definition_id: 2,
        valsi_id: 3,
        link_id: 4,
        translation: {} as any,
      },
    })

    const wrapper = mount(TranslationModal, {
      props: {
        show: true,
        sourceDefinition: mockSourceDefinition,
        languages: mockLanguages,
      },
    })

    await wrapper.find('input[type="text"]').setValue('coi')
    await wrapper.find('textarea').setValue('A greeting')
    await wrapper.find('form').trigger('submit')

    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('success')).toBeTruthy()
    expect(wrapper.emitted('success')?.[0]).toEqual([2])
  })
})
```

---

## Deployment Checklist

### Pre-deployment
- [ ] All tests passing
- [ ] Code review completed
- [ ] Database migration tested on staging
- [ ] API documentation updated
- [ ] Frontend translations added for all languages
- [ ] Performance testing completed
- [ ] Security review completed

### Deployment
- [ ] Run database migration
- [ ] Deploy backend changes
- [ ] Deploy frontend changes
- [ ] Verify health checks
- [ ] Test translation workflow in production
- [ ] Monitor error logs

### Post-deployment
- [ ] Announce feature to users
- [ ] Monitor usage metrics
- [ ] Gather user feedback
- [ ] Create follow-up tasks for improvements

---

## Monitoring & Metrics

### Key Metrics to Track

```sql
-- Translation creation rate
SELECT 
  DATE(created_at) as date,
  COUNT(*) as translations_created
FROM definition_links
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- Most translated phrases
SELECT 
  v.word,
  COUNT(DISTINCT dl.translation_id) as translation_count
FROM definition_links dl
JOIN definitions d ON dl.definition_id = d.definitionid
JOIN valsi v ON d.valsiid = v.valsiid
GROUP BY v.word
ORDER BY translation_count DESC
LIMIT 20;

-- Translation completion time (requires tracking)
SELECT 
  AVG(EXTRACT(EPOCH FROM (dl.created_at - d.created_at))) as avg_seconds
FROM definition_links dl
JOIN definitions d ON dl.definition_id = d.definitionid
WHERE dl.created_at > NOW() - INTERVAL '7 days';
```

### Error Monitoring

```typescript
// Frontend error tracking
window.addEventListener('unhandledrejection', (event) => {
  if (event.reason?.config?.url?.includes('/translate')) {
    // Log translation errors to monitoring service
    console.error('Translation error:', event.reason)
  }
})
```

---

## Troubleshooting

### Common Issues

**Issue:** "Source definition must be a phrase"
- **Cause:** Trying to translate non-phrase definition
- **Solution:** Only phrases (typeid=15) can be translated

**Issue:** Duplicate valsi created
- **Cause:** Race condition in concurrent requests
- **Solution:** Use database-level uniqueness constraint

**Issue:** Translation not appearing in list
- **Cause:** Cache not invalidated
- **Solution:** Clear cache after translation creation

**Issue:** Modal not opening
- **Cause:** Languages not loaded
- **Solution:** Ensure languages are fetched before showing modal

---

**Document Version:** 1.0  
**Last Updated:** 2026-05-13  
**Author:** Claude (Kiro AI)
