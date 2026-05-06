<template>
  <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
    {{
      isEditMode
        ? t('upsertDefinition.editTitle')
        : prefilledWord
          ? t('upsertDefinition.addTitle')
          : t('upsertDefinition.addEntryTitle')
    }}
  </h2>

  <form class="space-y-4 sm:space-y-6" @submit.prevent="submitValsi">
    <!-- Word Input and Analysis -->
    <div>
      <label for="word" class="block text-sm font-medium text-blue-700"
        >{{ t('upsertDefinition.wordLabel') }}
        <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label
      >
      <div class="flex flex-col sm:flex-row gap-2 sm:space-x-2">
        <div class="flex-1 w-full">
          <!-- Added wrapper div with flex-1 -->
          <DynamicInput
            id="word"
            v-model="word"
            :is-analyzing="isAnalyzing"
            :is-submitting="isSubmitting"
            :prefilled-word="prefilledWord"
            :is-edit-mode="isEditMode"
            @clear-analysis="clearAnalysis"
          />
        </div>
        <!-- Only show Analyze button when adding new word -->
        <div class="flex items-center justify-end">
          <Button
            v-if="!isEditMode"
            variant="warning-orange"
            size="lg"
            class="w-auto"
            :disabled="isAnalyzing || isSubmitting || word === ''"
            :loading="isAnalyzing"
            @click="doAnalyzeWord"
          >
            <template #icon> <Search class="h-6 w-6 shrink-0" aria-hidden="true" /> </template>
            {{ t('upsertDefinition.analyzeButton') }}
          </Button>
        </div>
      </div>
    </div>
    <!-- Word Type Display -->
    <div v-if="!isEditMode && wordType" class="space-y-4">
      <AlertComponent type="info" :label="t('upsertDefinition.detectedTypeLabel')">
        <p class="font-semibold">{{ wordType }}</p>
      </AlertComponent>
      <AlertComponent
        v-if="recommended"
        type="tip"
        :label="t('upsertDefinition.recommendedWordLabel')"
      >
        <div class="flex items-center gap-2 justify-start">
          <h2 class="font-semibold truncate">{{ recommended }}</h2>
          <button type="button" class="ui-btn--edit" @click="useRecommended">
            <ArrowRight class="h-4 w-4" /> {{ t('upsertDefinition.useThisButton') }}
          </button>
        </div>
      </AlertComponent>
      <div v-if="problems" class="space-y-4">
        <div v-for="(issues, category) in problems" :key="category">
          <AlertComponent
            v-if="issues.length > 0"
            type="error"
            :label="
              category === 'regular'
                ? t('upsertDefinition.similarRegularGismu')
                : t('upsertDefinition.similarExperimentalGismu')
            "
          >
            <ul class="list-disc list-inside space-y-1">
              <li v-for="(problem, index) in issues" :key="index" class="font-semibold truncate">
                {{ problem }}
              </li>
            </ul>
          </AlertComponent>
        </div>
      </div>
    </div>
    <!-- Combined Language Selectors -->
    <div class="flex flex-col sm:flex-row gap-4">
      <!-- Optional Entry Language Selection (Only for new entries) -->
      <div class="flex-1">
        <label for="source-language" class="block text-sm font-medium text-blue-700"
          >{{ t('upsertDefinition.sourceLanguageLabel') }}
          <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label
        >
        <select
          id="source-language"
          v-model="sourceLangId"
          required
          class="input-field w-full h-10"
          :disabled="isLoading || isSubmitting || isEditMode || prefilledWord"
          :readonly="prefilledWord || isEditMode"
        >
          <!-- Default Lojban option -->
          <option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.real_name }}
          </option>
        </select>
        <p class="mt-1 text-xs text-gray-500">
          {{
            t(
              'upsertDefinition.sourceLanguageNote',
              `The language the word itself belongs to. Cannot be changed after
          creation.`
            )
          }}
        </p>
      </div>
      <!-- Language Selection -->
      <div class="flex-1">
        <label for="language" class="block text-sm font-medium text-blue-700"
          >{{ t('upsertDefinition.languageLabel') }}
          <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label
        >
        <select
          id="language"
          v-model="langId"
          required
          class="input-field w-full h-10"
          :class="{
            'border-red-500 focus:ring-red-500 focus:border-red-500':
              shouldHighlightMissing && missingFields.langId,
          }"
          :disabled="isLoading || isSubmitting"
        >
          <option value="">{{ t('upsertDefinition.selectLanguagePlaceholder') }}</option>

          <option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.real_name }}
          </option>
        </select>
      </div>
    </div>
    <!-- Rafsi Input (Lojban only) -->
    <div v-if="Number(sourceLangId) === 1">
      <label for="rafsi" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.rafsiLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <input
        id="rafsi"
        v-model="rafsi"
        type="text"
        :placeholder="t('upsertDefinition.rafsiPlaceholder')"
        class="input-field w-full"
        :disabled="isSubmitting"
      />
      <p class="mt-1 text-xs text-gray-500">{{ t('upsertDefinition.rafsiNote') }}</p>
    </div>
    <!-- Selmaho Input (cmavo / experimental cmavo only) -->
    <div v-if="showSelmahoField">
      <label for="selmaho" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.selmahoLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <input
        id="selmaho"
        v-model="selmaho"
        type="text"
        :placeholder="t('upsertDefinition.selmahoPlaceholder')"
        class="input-field w-full"
        :disabled="isSubmitting"
      />
      <p class="mt-1 text-xs text-gray-500">{{ t('upsertDefinition.selmahoNote') }}</p>
    </div>
    <!-- Definition Input -->
    <div>
      <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
        <div class="flex flex-wrap items-center gap-2">
          <label for="definition" class="block text-sm font-medium text-blue-700"
            >{{ t('upsertDefinition.definitionLabel') }}
            <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label
          >
          <button
            type="button"
            class="ui-btn--empty inline-flex items-center gap-1 text-xs py-0.5 px-2"
            :title="t('upsertDefinition.previewDefinitionTitle')"
            @click="openDefinitionPreview"
          >
            <Eye class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
            {{ t('upsertDefinition.previewButton') }}
          </button>
        </div>
        <span class="text-xs text-gray-500">{{ t('upsertDefinition.requiredUnlessImage') }}</span>
      </div>
      <textarea
        id="definition"
        v-model="definition"
        :required="!imageData"
        rows="4"
        :class="{
          'textarea-field': true,
          'border-red-300 focus:ring-red-500 focus:border-red-500': definitionError,
          'border-red-500 focus:ring-red-500 focus:border-red-500':
            shouldHighlightMissing && missingFields.definition && !definitionError,
          'border-blue-300 focus:ring-blue-500 focus:border-blue-500':
            !definitionError && !(shouldHighlightMissing && missingFields.definition),
        }"
        :disabled="isSubmitting"
      />
      <p v-if="definitionError" class="mt-2 text-xs sm:text-sm text-red-600">
        {{ definitionError }}
      </p>

      <p v-if="!definitionError" class="mt-2 text-xs sm:text-sm text-gray-500">
        {{ t('upsertDefinition.mathjaxNote') }}
      </p>
    </div>

    <div>
      <ImageUpload
        v-model="imageData"
        :definition-id="editDefinitionId"
        :has-existing-image="hasImage"
        :note="t('upsertDefinition.requiredUnlessDefinitionProvided')"
        @image-loaded="handleImageLoaded"
        @remove-image="handleRemoveImage"
      />
    </div>
    <!-- Notes Input -->
    <div>
      <div class="flex flex-wrap items-center gap-2 mb-1">
        <label for="notes" class="block text-sm font-medium text-blue-700">
          {{ t('upsertDefinition.notesLabel') }}
          <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
        </label>
        <button
          type="button"
          class="ui-btn--empty inline-flex items-center gap-1 text-xs py-0.5 px-2"
          :title="t('upsertDefinition.previewNotesTitle')"
          @click="openNotesPreview"
        >
          <Eye class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
          {{ t('upsertDefinition.previewButton') }}
        </button>
      </div>
      <textarea
        id="notes"
        v-model="notes"
        rows="3"
        class="textarea-field"
        :disabled="isSubmitting"
      />
    </div>
    <!-- Etymology Input -->
    <div>
      <label for="etymology" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.etymologyLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <textarea
        id="etymology"
        v-model="etymology"
        rows="3"
        class="textarea-field"
        :disabled="isSubmitting"
      />
    </div>
    <!-- Jargon Input -->
    <div>
      <label for="jargon" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.jargonLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <textarea
        id="jargon"
        v-model="jargon"
        rows="2"
        class="textarea-field"
        :disabled="isSubmitting"
      />
    </div>
    <!-- Gloss Keywords -->
    <div>
      <label class="block text-sm font-medium text-blue-700 mb-2">
        {{ t('upsertDefinition.glossKeywordsLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <div
        v-for="(keyword, index) in glossKeywords"
        :key="'gloss' + index"
        class="flex flex-col sm:flex-row gap-2 sm:space-x-2 mb-2 items-center"
      >
        <input
          v-model="keyword.word"
          type="text"
          :placeholder="t('upsertDefinition.keywordPlaceholder')"
          class="flex-1 input-field w-full"
        />
        <input
          v-model="keyword.meaning"
          type="text"
          :placeholder="t('upsertDefinition.meaningPlaceholder')"
          class="flex-1 input-field w-full"
        />
        <div class="flex flex-wrap space-x-2">
          <button type="button" class="sm:w-auto ui-btn--delete" @click="removeGlossKeyword(index)">
            <CircleMinus class="h-4 w-4" /> {{ t('upsertDefinition.removeButton') }}
          </button>
          <button type="button" class="ui-btn--neutral" @click="addGlossKeyword">
            <CirclePlus class="h-4 w-4" /> {{ t('upsertDefinition.addGlossButton') }}
          </button>
        </div>
      </div>
    </div>
    <!-- Place Keywords -->
    <div>
      <label class="block text-sm font-medium text-blue-700 mb-2">
        {{ t('upsertDefinition.placeKeywordsLabel') }}
        <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span>
      </label>
      <div
        v-for="(keyword, index) in placeKeywords"
        :key="'place' + index"
        class="flex flex-col sm:flex-row gap-2 sm:space-x-2 mb-2 items-center"
      >
        <input
          v-model="keyword.word"
          type="text"
          :placeholder="t('upsertDefinition.keywordPlaceholder')"
          class="flex-1 input-field w-full"
        />
        <input
          v-model="keyword.meaning"
          type="text"
          :placeholder="t('upsertDefinition.meaningPlaceholder')"
          class="flex-1 input-field w-full"
        />
        <div class="flex flex-wrap space-x-2">
          <button type="button" class="sm:w-auto ui-btn--delete" @click="removePlaceKeyword(index)">
            <CircleMinus class="h-4 w-4" /> {{ t('upsertDefinition.removeButton') }}
          </button>
          <button type="button" class="w-auto ui-btn--neutral" @click="addPlaceKeyword">
            <CirclePlus class="h-4 w-4" /> {{ t('upsertDefinition.addPlaceButton') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="!isEditMode || (isEditMode && isAuthor)" class="mb-4">
      <label class="flex items-center space-x-2">
        <input v-model="ownerOnly" type="checkbox" class="checkbox-toggle" />
        <span class="text-xs sm:text-sm text-gray-700"
          >{{ t('upsertDefinition.ownerOnlyLabel') }}
          <span class="text-gray-500">{{ t('upsertDefinition.optional') }}</span></span
        >
      </label>
      <p class="mt-1 text-xs sm:text-sm text-gray-500">
        {{ t('upsertDefinition.ownerOnlyNote') }}
      </p>
    </div>
    <!-- Submit / Analyze Button -->
    <div class="flex justify-center w-full">
      <!-- Show Submit button if form is valid and not submitting -->
      <Button
        v-if="isValid && !isSubmitting"
        type="submit"
        size="lg"
        class="max-w-fit ui-btn--create h-10 text-base"
      >
        {{
          isEditMode
            ? t('upsertDefinition.updateButton')
            : prefilledWord
              ? t('upsertDefinition.addButton')
              : t('upsertDefinition.addEntryButton')
        }}
      </Button>
      <!-- Show Analyze button if form is invalid and not submitting -->
      <Button
        v-else-if="!isValid && !isSubmitting"
        variant="warning-orange"
        size="lg"
        class="max-w-fit"
        :disabled="isAnalyzing || word === ''"
        :loading="isAnalyzing"
        @click="analyzeAndScroll"
      >
        <template #icon> <Search class="h-6 w-6 shrink-0" aria-hidden="true" /> </template>
        {{ t('upsertDefinition.analyzeWordButton') }}
      </Button>
      <!-- Show disabled state during submission -->
      <Button
        v-else-if="isSubmitting"
        type="button"
        size="lg"
        class="max-w-fit ui-btn--create h-10 text-base"
        disabled
      >
        {{
          isEditMode
            ? isSubmitting
              ? t('upsertDefinition.updating')
              : t('upsertDefinition.updateButton')
            : prefilledWord
              ? isSubmitting
                ? t('upsertDefinition.adding')
                : t('upsertDefinition.addButton')
              : isSubmitting
                ? t('upsertDefinition.adding')
                : t('upsertDefinition.addEntryButton')
        }}
      </Button>
    </div>
  </form>
  <ModalComponent :show="previewKind !== null" :title="previewModalTitle" @close="closePreview">
    <div
      v-if="previewKind === 'definition'"
      class="text-sm prose prose-sm max-w-none text-gray-700 overflow-y-auto"
    >
      <LazyMathJax v-if="definition.trim()" :content="definition" />
      <p v-else class="text-gray-500 text-sm">{{ t('upsertDefinition.previewEmpty') }}</p>
    </div>

    <div
      v-else-if="previewKind === 'notes'"
      class="w-full text-sm text-gray-600 bg-gray-100 p-2 rounded-md overflow-y-auto"
    >
      <h4 class="italic text-gray-600">{{ t('upsertDefinition.notesLabel') }}</h4>
      <LazyMathJax v-if="notes.trim()" :content="notes" :enable-markdown="true" />
      <p v-else class="text-gray-500 text-sm">{{ t('upsertDefinition.previewEmpty') }}</p>
    </div>
  </ModalComponent>
</template>

<script setup lang="ts">
import { ArrowRight, Search, CirclePlus, CircleMinus, Eye } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import {
  addValsi,
  updateValsi,
  analyzeWord,
  getLanguages,
  validateMathJax,
  getDefinition,
  linkDefinitions,
} from '@/api'
import { Button } from '@packages/ui'
import AlertComponent from '@/components/AlertComponent.vue'
import DynamicInput from '@/components/DynamicInput.vue'
import ImageUpload from '@/components/ImageUpload.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'

const props = defineProps({
  id: {
    type: String,
    required: false,
    default: null,
  },
})

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()
const { showSuccess } = useSuccessToast()
const { t } = useI18n()

/** Format API error for display; e.g. RAFSI_CONFLICT|word|type -> translated message; LaTeX field errors -> localized */
function formatDefinitionError(apiError: unknown): string {
  if (typeof apiError !== 'string') return String(apiError ?? '')
  if (apiError.startsWith('RAFSI_CONFLICT|')) {
    const parts = apiError.split('|')
    const word = parts[1] ?? ''
    const type = parts[2] ?? ''
    return t('upsertDefinition.rafsiConflict', { word, type })
  }
  const latexPrefix = 'Invalid LaTeX/MathJax in '
  if (apiError.startsWith(latexPrefix)) {
    const after = apiError.slice(latexPrefix.length)
    const colon = after.indexOf(': ')
    const fieldKey = colon >= 0 ? after.slice(0, colon).trim() : after
    const details = colon >= 0 ? after.slice(colon + 2).trim() : ''
    const fieldLabel =
      fieldKey === 'definition'
        ? t('upsertDefinition.definitionLabel')
        : fieldKey === 'notes'
          ? t('upsertDefinition.notesLabel')
          : fieldKey === 'etymology'
            ? t('upsertDefinition.etymologyLabel')
            : fieldKey
    return t('upsertDefinition.validateErrorInField', { field: fieldLabel, details })
  }
  return apiError
}

// Form state
const word = ref('')
const recommended = ref('')
const problems = ref<Record<string, string[]>>({})
const wordId = ref('')
const langId = ref('')
const sourceLangId = ref(1)
const definition = ref('')
const rafsi = ref('')
const selmaho = ref('')
const notes = ref('')
const etymology = ref('')
const jargon = ref('')
const wordType = ref('')
const glossKeywords = ref([{ word: '', meaning: '' }])
const placeKeywords = ref([{ word: '', meaning: '' }])
const ownerOnly = ref(false)
const hasImage = ref(false)
const imageData = ref(null)
const removeImage = ref(false)

useSeoHead({ title: () => t('upsertDefinition.addEntryTitle'), robots: 'noindex, nofollow' })
// UI state
const definitionError = ref('')
const isAnalyzing = ref(false)
const isSubmitting = ref(false)
const isLoading = ref(true)
const prefilledWord = ref(false)
const isEditMode = ref(false)
const isAuthor = ref(false)
const editDefinitionId = ref(null)
/** Matches DefinitionCard: definition = MathJax only; notes = markdown + MathJax */
const previewKind = ref<null | 'definition' | 'notes'>(null)

const previewModalTitle = computed(() => {
  if (previewKind.value === 'definition') return t('upsertDefinition.previewDefinitionTitle')
  if (previewKind.value === 'notes') return t('upsertDefinition.previewNotesTitle')
  return ''
})

const openDefinitionPreview = () => {
  previewKind.value = 'definition'
}

const openNotesPreview = () => {
  previewKind.value = 'notes'
}

const closePreview = () => {
  previewKind.value = null
}

const handleRemoveImage = () => {
  hasImage.value = false
  removeImage.value = true
  imageData.value = null
}

const handleImageLoaded = (imageObj) => {
  imageData.value = imageObj
}

// Load existing definition data
const loadDefinitionData = async (definitionId: string | number) => {
  try {
    const response = await getDefinition(definitionId)
    const def = response.data

    if (def) {
      word.value = def.valsiword
      wordId.value = def.valsiid
      langId.value = def.langid
      definition.value = def.definition
      rafsi.value = def.rafsi || ''
      selmaho.value = def.selmaho || ''
      notes.value = def.notes || ''
      etymology.value = def.etymology || ''
      jargon.value = def.jargon || ''
      wordType.value = def.type_name
      ownerOnly.value = def.owner_only
      isAuthor.value = auth.state.username === def.username
      hasImage.value = def.has_image

      // Load keywords from the response
      glossKeywords.value =
        def.gloss_keywords.length > 0 ? def.gloss_keywords : [{ word: '', meaning: '' }]

      placeKeywords.value =
        def.place_keywords.length > 0 ? def.place_keywords : [{ word: '', meaning: '' }]

      prefilledWord.value = true
    }
  } catch (err) {
    showError(t('upsertDefinition.loadDefinitionError'))
    console.error('Error loading definition:', err)
  }
}

// Data
const languages = ref([])
const validationTimeout = ref(null)

// Computed
const isValid = computed(
  () =>
    word.value &&
    langId.value &&
    (definition.value || imageData.value) &&
    wordType.value &&
    !definitionError.value
)

// Track missing required fields for highlighting
const missingFields = computed((): Record<string, boolean> => {
  const missing: Record<string, boolean> = {}
  // Only show missing fields if wordType is set (analysis was done)
  if (wordType.value) {
    if (!langId.value) missing.langId = true
    if (!definition.value && !imageData.value) missing.definition = true
  }
  return missing
})

// Check if we should highlight fields (form invalid after analysis)
const shouldHighlightMissing = computed(() => {
  return wordType.value && !isValid.value
})

// Show selmaho field for cmavo and experimental cmavo
const showSelmahoField = computed(() => {
  const t = wordType.value || ''
  return t === 'cmavo' || t === 'experimental cmavo'
})

// Methods for keywords
const addGlossKeyword = () => {
  glossKeywords.value.push({ word: '', meaning: '' })
}

const removeGlossKeyword = (index: number) => {
  glossKeywords.value.splice(index, 1)
  if (glossKeywords.value.length === 0) {
    glossKeywords.value.push({ word: '', meaning: '' })
  }
}

const addPlaceKeyword = () => {
  placeKeywords.value.push({ word: '', meaning: '' })
}

const removePlaceKeyword = (index: number) => {
  placeKeywords.value.splice(index, 1)
  if (placeKeywords.value.length === 0) {
    placeKeywords.value.push({ word: '', meaning: '' })
  }
}

const LAST_LANG_KEY = 'lastSelectedLanguage'

const setLastLanguage = (langId: string) => {
  if (typeof window === 'undefined') return

  localStorage.setItem(LAST_LANG_KEY, langId)
}

const getLastLanguage = () => {
  if (typeof window === 'undefined') return

  return localStorage.getItem(LAST_LANG_KEY)
}

const loadLanguages = async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data

    const lastLang = getLastLanguage()
    if (lastLang) {
      langId.value = lastLang
    }
  } catch (e) {
    showError(e.response?.data?.error || t('upsertDefinition.loadLanguagesError'))
  } finally {
    isLoading.value = false
  }
}

watch(langId, (newValue) => {
  if (newValue) {
    setLastLanguage(newValue)
  }
})

// Initialization
onMounted(async () => {
  const rawRouteId = props.id ?? route.params.id
  const definitionId = Array.isArray(rawRouteId) ? rawRouteId[0] : rawRouteId
  if (definitionId) {
    isEditMode.value = true
    useSeoHead({ title: () => 'Updating entry', robots: 'noindex, nofollow' })
    editDefinitionId.value = definitionId
    await loadDefinitionData(definitionId) // This will set sourceLangId from loaded data
  } else {
    const wordFromUrl = queryStr(route.query.word)
    if (wordFromUrl) {
      word.value = decodeURIComponent(wordFromUrl)
      prefilledWord.value = true
      await doAnalyzeWord() // Analyze prefilled word
    } else {
      sourceLangId.value = 1 // Default for completely new entry
    }
    isAuthor.value = true
  }
  await loadLanguages() // Load languages after potentially setting sourceLangId

  // Handle translation pre-filling
  const translateFromId = queryStr(route.query.translate_from_def)
  if (translateFromId && !isEditMode.value) {
    try {
      const res = await getDefinition(translateFromId)
      const sourceDef = res.data
      // Pre-fill fields that might be useful
      // For now, we mainly want to ensure we don't copy the definition/language blindly
      // But maybe copying notes/keywords is helpful?
      // Let's just set the word if not already set (though it should be set by word query param)
      if (!word.value && sourceDef.valsiword) {
        word.value = sourceDef.valsiword
        prefilledWord.value = true
      }
      useSeoHead({
        title: () => t('upsertDefinition.addTranslationTitle'),
        robots: 'noindex, nofollow',
      })
    } catch (e) {
      console.error('Error loading source definition for translation:', e)
    }
  }
})

const clearAnalysis = () => {
  if (!prefilledWord.value) {
    wordType.value = ''
    recommended.value = ''
    problems.value = {}
    clearError()
  }
}

const useRecommended = () => {
  if (recommended.value) {
    word.value = recommended.value
    recommended.value = ''
  }
}

const doAnalyzeWord = async () => {
  if (!word.value) return
  word.value = word.value.trim()
  isAnalyzing.value = true

  try {
    const response = await analyzeWord(word.value)
    if (response.data?.success) {
      clearError()
      wordType.value = response.data.word_type
      word.value = response.data.text
      recommended.value =
        response.data.recommended && response.data.recommended !== word.value
          ? response.data.recommended
          : ''
      problems.value = response.data.problems || {}
    } else {
      wordType.value = ''
      showError(t('upsertDefinition.analyzeError'))
    }
  } catch {
    showError(t('upsertDefinition.analyzeErrorGeneric'))
  } finally {
    isAnalyzing.value = false
  }
}

const performValidateMathJax = async () => {
  if (!definition.value) {
    definitionError.value = ''
    return
  }

  try {
    const response = await validateMathJax(definition.value)

    if (response.data.valid) {
      definitionError.value = ''
    }
  } catch (err) {
    definitionError.value = err.response?.data?.error || t('upsertDefinition.validateError')
  }
}

// Modified submit handler
const submitValsi = async () => {
  if (!isValid.value) return

  try {
    await performValidateMathJax()
    if (definitionError.value) return

    isSubmitting.value = true
    clearError()

    // Filter out empty keywords
    const filteredGlossKeywords = glossKeywords.value.filter((k) => k.word.trim() !== '')
    const filteredPlaceKeywords = placeKeywords.value.filter((k) => k.word.trim() !== '')

    const requestData: Record<string, unknown> = {
      word: word.value,
      definition: definition.value,
      rafsi: rafsi.value || null,
      ...(showSelmahoField.value && { selmaho: selmaho.value?.trim() || null }),
      notes: notes.value || null,
      etymology: etymology.value || null,
      jargon: jargon.value || null,
      lang_id: parseInt(langId.value, 10),
      owner_only: ownerOnly.value,
      image: imageData.value,
      remove_image: removeImage.value,
      // Always include source_langid when adding a new definition (defaults to 1)
      ...(!isEditMode.value && { source_langid: parseInt(String(sourceLangId.value), 10) || 1 }),
    }

    // Include keywords if they're not empty
    if (filteredGlossKeywords.length > 0) {
      requestData.gloss_keywords = filteredGlossKeywords
    }
    if (filteredPlaceKeywords.length > 0) {
      requestData.place_keywords = filteredPlaceKeywords
    }

    let response
    if (isEditMode.value) {
      response = await updateValsi(editDefinitionId.value, requestData)
    } else {
      response = await addValsi(requestData)
    }

    if (response.data.success) {
      const existingWord = response.data.existing_word || false
      if (isEditMode.value) {
        showSuccess(t('upsertDefinition.updateSuccess'))
      } else if (existingWord) {
        showSuccess(t('upsertDefinition.existingWordNote'))
      } else {
        showSuccess(t('upsertDefinition.createSuccess'))
      }

      const newDefinitionId = response.data.definition_id || editDefinitionId.value

      // Auto-link if translation source is present
      const sourceDefId = queryStr(route.query.translate_from_def)
      if (sourceDefId && !isEditMode.value) {
        try {
          await linkDefinitions(sourceDefId, newDefinitionId)
        } catch (linkError) {
          console.error('Failed to auto-link definitions:', linkError)
          showError(t('upsertDefinition.linkError'))
        }
      }

      // Brief delay before redirect so the success toast is visible
      setTimeout(() => {
        router.push(
          `/valsi/${word.value.replace(/ /g, '_')}?highlight_definition_id=${newDefinitionId}`
        )
      }, 1500)
    } else {
      showError(formatDefinitionError(response.data.error) || t('upsertDefinition.saveError'))
      definitionError.value = ''
    }
  } catch (e) {
    showError(
      formatDefinitionError(e.response?.data?.error) || t('upsertDefinition.saveErrorGeneric')
    )
    isSubmitting.value = false
  }
}

// Watch for definition changes to validate MathJax
watch(definition, () => {
  if (validationTimeout.value) {
    clearTimeout(validationTimeout.value)
  }
  validationTimeout.value = setTimeout(() => {
    performValidateMathJax()
  }, 500)
})

const analyzeAndScroll = async () => {
  await doAnalyzeWord()
  const mainContent = document.querySelector('.main-content')
  if (mainContent) {
    mainContent.scrollTo({ top: 0, behavior: 'smooth' })
  }
}
</script>
