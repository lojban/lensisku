<template>
  <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
    {{ isEditMode ? t('upsertDefinition.editTitle') : prefilledWord ? t('upsertDefinition.addTitle') :
      t('upsertDefinition.addEntryTitle') }}
  </h2>

  <form class="space-y-4 sm:space-y-6" @submit.prevent="submitValsi">
    <!-- Word Input and Analysis -->
    <div>
      <label for="word" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.wordLabel') }} <span
          class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
      <div class="flex flex-col sm:flex-row gap-2 sm:space-x-2">
        <div class="flex-1 w-full">
          <!-- Added wrapper div with flex-1 -->
          <DynamicInput id="word" v-model="word" :is-analyzing="isAnalyzing" :is-submitting="isSubmitting"
            :prefilled-word="prefilledWord" :is-edit-mode="isEditMode" @clear-analysis="clearAnalysis" />
        </div>
        <!-- Only show Analyze button when adding new word -->
        <div class="flex items-center justify-end">
          <button v-if="!isEditMode" type="button" class="w-auto h-8 btn-aqua-orange text-base"
            :disabled="isAnalyzing || isSubmitting || word === ''" @click="doAnalyzeWord">
            <div class="flex items-center gap-2">
              <Loader v-if="isAnalyzing" class="h-4 w-4 animate-spin" />
              <Search v-else class="h-4 w-4" />
              <span>{{ t('upsertDefinition.analyzeButton') }}</span>
            </div>
          </button>
        </div>
      </div>
    </div>

    <!-- Word Type Display -->
    <div v-if="!isEditMode && wordType" class="space-y-4">
      <AlertComponent type="info" :label="t('upsertDefinition.detectedTypeLabel')">
        <p class="font-semibold">
          {{ wordType }}
        </p>
      </AlertComponent>

      <AlertComponent v-if="recommended" type="tip" :label="t('upsertDefinition.recommendedWordLabel')">
        <div class="flex items-center gap-2 justify-start">
          <h2 class="font-semibold truncate">
            {{ recommended }}
          </h2>
          <button type="button" class="btn-update" @click="useRecommended">
            <ArrowRight class="h-4 w-4" />
            {{ t('upsertDefinition.useThisButton') }}
          </button>
        </div>
      </AlertComponent>

      <div v-if="problems" class="space-y-4">
        <div v-for="(issues, category) in problems" :key="category">
          <AlertComponent v-if="issues.length > 0" type="error"
            :label="category === 'regular' ? t('upsertDefinition.similarRegularGismu') : t('upsertDefinition.similarExperimentalGismu')">
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
          <label for="source-language" class="block text-sm font-medium text-blue-700">{{
            t('upsertDefinition.sourceLanguageLabel', 'Entry Language') }} <span class="text-red-500">{{
              t('upsertDefinition.required') }}</span></label>
          <select id="source-language" v-model="sourceLangId" required class="input-field w-full h-10"
            :disabled="isLoading || isSubmitting || isEditMode || prefilledWord"
            :readonly="prefilledWord || isEditMode">
            <!-- Default Lojban option -->
            <option v-for="lang in languages" :key="lang.id" :value="lang.id">
              {{ lang.real_name }} ({{ lang.english_name }})
            </option>
          </select>
          <p class="mt-1 text-xs text-gray-500">
            {{ t('upsertDefinition.sourceLanguageNote', `The language the word itself belongs to. Cannot be changed after
            creation.`) }}
          </p>
      </div>

      <!-- Language Selection -->
      <div class="flex-1">
        <label for="language" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.languageLabel') }}
          <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
        <select id="language" v-model="langId" required class="input-field w-full h-10"
          :class="{
            'border-red-500 focus:ring-red-500 focus:border-red-500': shouldHighlightMissing && missingFields.langId
          }"
          :disabled="isLoading || isSubmitting">
          <option value="">
            {{ t('upsertDefinition.selectLanguagePlaceholder') }}
          </option>
          <option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.real_name }} ({{ lang.english_name }})
          </option>
        </select>
      </div>
    </div>

    <!-- Definition Input -->
    <div>
      <div class="flex items-center justify-between">
        <label for="definition" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.definitionLabel')
          }} <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
        <span class="text-xs text-gray-500">{{ t('upsertDefinition.requiredUnlessImage') }}</span>
      </div>
      <textarea id="definition" v-model="definition" :required="!imageData" rows="4" :class="{
        'textarea-field': true,
        'border-red-300 focus:ring-red-500 focus:border-red-500': definitionError,
        'border-red-500 focus:ring-red-500 focus:border-red-500': shouldHighlightMissing && missingFields.definition && !definitionError,
        'border-blue-300 focus:ring-blue-500 focus:border-blue-500': !definitionError && !(shouldHighlightMissing && missingFields.definition),
      }" :disabled="isSubmitting" />
      <p v-if="definitionError" class="mt-2 text-xs sm:text-sm text-red-600">
        {{ definitionError }}
      </p>
      <p v-if="!definitionError" class="mt-2 text-xs sm:text-sm text-gray-500">
        {{ t('upsertDefinition.mathjaxNote') }}
      </p>
    </div>

    <div>
      {{ imageData }}
      <ImageUpload v-model="imageData" :definition-id="editDefinitionId" :has-existing-image="hasImage"
        :note="t('upsertDefinition.requiredUnlessDefinitionProvided')" @image-loaded="handleImageLoaded"
        @remove-image="handleRemoveImage" />
    </div>
    <!-- Notes Input -->
    <div>
      <label for="notes" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.notesLabel') }} <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional')
          }}</span>
      </label>
      <textarea id="notes" v-model="notes" rows="3" class="textarea-field" :disabled="isSubmitting" />
    </div>

    <!-- Etymology Input -->
    <div>
      <label for="etymology" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.etymologyLabel') }} <span class="text-gray-500 font-normal">{{
          t('upsertDefinition.optional') }}</span>
      </label>
      <textarea id="etymology" v-model="etymology" rows="3" class="textarea-field" :disabled="isSubmitting" />
    </div>

    <!-- Jargon Input -->
    <div>
      <label for="jargon" class="block text-sm font-medium text-blue-700">
        {{ t('upsertDefinition.jargonLabel', 'Jargon') }} <span class="text-gray-500 font-normal">{{
          t('upsertDefinition.optional') }}</span>
      </label>
      <textarea id="jargon" v-model="jargon" rows="2" class="textarea-field" :disabled="isSubmitting" />
    </div>

    <!-- Gloss Keywords -->
    <div>
      <label class="block text-sm font-medium text-blue-700 mb-2">
        {{ t('upsertDefinition.glossKeywordsLabel') }} <span class="text-gray-500 font-normal">{{
          t('upsertDefinition.optional') }}</span>
      </label>
      <div v-for="(keyword, index) in glossKeywords" :key="'gloss' + index"
        class="flex flex-col sm:flex-row gap-2 sm:space-x-2 mb-2 items-center">
        <input v-model="keyword.word" type="text" :placeholder="t('upsertDefinition.keywordPlaceholder')"
          class="flex-1 input-field w-full">
        <input v-model="keyword.meaning" type="text" :placeholder="t('upsertDefinition.meaningPlaceholder')"
          class="flex-1 input-field w-full">
        <div class="flex flex-wrap space-x-2">
          <button type="button" class="sm:w-auto btn-aqua-red" @click="removeGlossKeyword(index)">
            <CircleMinus class="h-4 w-4" />
            {{ t('upsertDefinition.removeButton') }}
          </button>
          <button type="button" class="btn-aqua-white" @click="addGlossKeyword">
            <CirclePlus class="h-4 w-4" />
            {{ t('upsertDefinition.addGlossButton') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Place Keywords -->
    <div>
      <label class="block text-sm font-medium text-blue-700 mb-2">
        {{ t('upsertDefinition.placeKeywordsLabel') }} <span class="text-gray-500 font-normal">{{
          t('upsertDefinition.optional') }}</span>
      </label>
      <div v-for="(keyword, index) in placeKeywords" :key="'place' + index"
        class="flex flex-col sm:flex-row gap-2 sm:space-x-2 mb-2 items-center">
        <input v-model="keyword.word" type="text" :placeholder="t('upsertDefinition.keywordPlaceholder')"
          class="flex-1 input-field w-full">
        <input v-model="keyword.meaning" type="text" :placeholder="t('upsertDefinition.meaningPlaceholder')"
          class="flex-1 input-field w-full">
        <div class="flex flex-wrap space-x-2">
          <button type="button" class="sm:w-auto btn-aqua-red" @click="removePlaceKeyword(index)">
            <CircleMinus class="h-4 w-4" />
            {{ t('upsertDefinition.removeButton') }}
          </button>
          <button type="button" class="w-auto btn-aqua-white" @click="addPlaceKeyword">
            <CirclePlus class="h-4 w-4" />
            {{ t('upsertDefinition.addPlaceButton') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="!isEditMode || (isEditMode && isAuthor)" class="mb-4">
      <label class="flex items-center space-x-2">
        <input v-model="ownerOnly" type="checkbox" class="checkbox-toggle">
        <span class="text-xs sm:text-sm text-gray-700">{{ t('upsertDefinition.ownerOnlyLabel') }} <span
            class="text-gray-500">{{ t('upsertDefinition.optional') }}</span></span>
      </label>
      <p class="mt-1 text-xs sm:text-sm text-gray-500">
        {{ t('upsertDefinition.ownerOnlyNote') }}
      </p>
    </div>

    <!-- Submit / Analyze Button -->
    <div class="flex justify-center w-full">
      <!-- Show Submit button if form is valid and not submitting -->
      <button v-if="isValid && !isSubmitting" type="submit" class="max-w-fit btn-aqua-emerald h-10 text-base">
        {{
          isEditMode
            ? t('upsertDefinition.updateButton')
            : prefilledWord
              ? t('upsertDefinition.addButton')
              : t('upsertDefinition.addEntryButton')
        }}
      </button>

      <!-- Show Analyze button if form is invalid and not submitting -->
      <button v-else-if="!isValid && !isSubmitting" type="button" class="max-w-fit btn-aqua-orange h-10 text-base"
        :disabled="isAnalyzing || word === ''" @click="analyzeAndScroll">
        <div class="flex items-center gap-2">
          <Loader v-if="isAnalyzing" class="h-4 w-4 animate-spin" />
          <Search v-else class="h-4 w-4" />
          <span>{{ t('upsertDefinition.analyzeWordButton') }}</span>
        </div>
      </button>

      <!-- Show disabled state during submission -->
      <button v-else-if="isSubmitting" type="button" class="max-w-fit btn-aqua-emerald h-10 text-base" disabled>
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
      </button>
    </div>

    <!-- Messages -->
    <div v-if="success" class="text-green-600 text-xs sm:text-sm mt-2">
      {{ isEditMode 
          && t('upsertDefinition.updateSuccess') }}
      <span v-if="isExistingWord && !isEditMode">{{ t('upsertDefinition.existingWordNote') }}</span>
    </div>
  </form>
</template>

<script setup>
import { ArrowRight, Search, Loader, CirclePlus, CircleMinus } from 'lucide-vue-next'
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
} from '@/api'
import AlertComponent from '@/components/AlertComponent.vue'
import DynamicInput from '@/components/DynamicInput.vue'
import ImageUpload from '@/components/ImageUpload.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

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
const { t, locale } = useI18n()

// Form state
const word = ref('')
const recommended = ref('')
const problems = ref({})
const wordId = ref('')
const langId = ref('')
const sourceLangId = ref(1)
const definition = ref('')
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

useSeoHead({ title: t("upsertDefinition.addEntryTitle") }, locale.value)
// UI state
const definitionError = ref('')
const success = ref(false)
const isAnalyzing = ref(false)
const isSubmitting = ref(false)
const isLoading = ref(true)
const isExistingWord = ref(false)
const prefilledWord = ref(false)
const isEditMode = ref(false)
const isAuthor = ref(false)
const editDefinitionId = ref(null)
const showSourceLanguageSelector = ref(false) // State to control visibility

const handleRemoveImage = () => {
  hasImage.value = false
  removeImage.value = true
  imageData.value = null
}

const handleImageLoaded = (imageObj) => {
  imageData.value = imageObj
}

// Load existing definition data
const loadDefinitionData = async (definitionId) => {
  try {
    const response = await getDefinition(definitionId)
    const def = response.data

    if (def) {
      word.value = def.valsiword
      wordId.value = def.valsiid
      langId.value = def.langid
      definition.value = def.definition
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
const missingFields = computed(() => {
  const missing = {}
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

// Methods for keywords
const addGlossKeyword = () => {
  glossKeywords.value.push({ word: '', meaning: '' })
}

const removeGlossKeyword = (index) => {
  glossKeywords.value.splice(index, 1)
  if (glossKeywords.value.length === 0) {
    glossKeywords.value.push({ word: '', meaning: '' })
  }
}

const addPlaceKeyword = () => {
  placeKeywords.value.push({ word: '', meaning: '' })
}

const removePlaceKeyword = (index) => {
  placeKeywords.value.splice(index, 1)
  if (placeKeywords.value.length === 0) {
    placeKeywords.value.push({ word: '', meaning: '' })
  }
}

const LAST_LANG_KEY = 'lastSelectedLanguage'

const setLastLanguage = (langId) => {
  if (typeof window === 'undefined') return;

  localStorage.setItem(LAST_LANG_KEY, langId)
}

const getLastLanguage = () => {
  if (typeof window === 'undefined') return;

  return localStorage.getItem(LAST_LANG_KEY)
}

const loadLanguages = async () => {
  try {
    const response = await getLanguages({})
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
  const definitionId = props.id || route.params.id;
  if (definitionId) {
    isEditMode.value = true;
    useSeoHead({ title: 'Updating entry' }, locale.value);
    editDefinitionId.value = definitionId;
    await loadDefinitionData(definitionId); // This will set sourceLangId from loaded data
  } else {
    const wordFromUrl = route.query.word;
    if (wordFromUrl) {
      word.value = decodeURIComponent(wordFromUrl);
      prefilledWord.value = true;
      await doAnalyzeWord(); // Analyze prefilled word
    } else {
      sourceLangId.value = 1; // Default for completely new entry
    }
    isAuthor.value = true;
  }
  await loadLanguages(); // Load languages after potentially setting sourceLangId
})

const clearAnalysis = () => {
  if (!prefilledWord.value) {
    wordType.value = ''
    recommended.value = ''
    problems.value = {}
    clearError()
    success.value = false
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
      success.value = true
      clearError()
      wordType.value = response.data.word_type
      word.value = response.data.text
      recommended.value = response.data.recommended && response.data.recommended !== word.value ? response.data.recommended : ''
      problems.value = response.data.problems || {};
    } else {
      success.value = false // Explicitly set success to false on API failure
      wordType.value = ''
      showError(t('upsertDefinition.analyzeError'))
    }
  } catch (e) {
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
    success.value = false
    isExistingWord.value = false

    // Filter out empty keywords
    const filteredGlossKeywords = glossKeywords.value.filter((k) => k.word.trim() !== '')
    const filteredPlaceKeywords = placeKeywords.value.filter((k) => k.word.trim() !== '')

    const requestData = {
      word: word.value,
      definition: definition.value,
      notes: notes.value || null,
      etymology: etymology.value || null,
      jargon: jargon.value || null,
      lang_id: parseInt(langId.value),
      owner_only: ownerOnly.value,
      image: imageData.value,
      remove_image: removeImage.value,
      // Always include source_langid when adding a new definition (defaults to 1)
      ...(!isEditMode.value && { source_langid: parseInt(sourceLangId.value) || 1 })
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
      success.value = true
      isExistingWord.value = response.data.existing_word || false

      // Wait a moment to show success message
      setTimeout(() => {
        const definitionId = response.data.definition_id || editDefinitionId.value
        router.push(`/valsi/${word.value}?highlight_definition_id=${definitionId}`)
      }, 1500)
    } else {
      showError(response.data.error || t('upsertDefinition.saveError'))
      definitionError.value = ''
    }
  } catch (e) {
    showError(e.response?.data?.error || t('upsertDefinition.saveErrorGeneric'))
  } finally {
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
