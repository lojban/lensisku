<template>

  <h1 class="text-2xl font-bold text-gray-800"> {{ t('dictionaryExport.title') }} </h1>

  <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-between my-4">

    <p class="text-gray-600"> {{ t('dictionaryExport.description') }} </p>
     <RouterLink to="/export/cached" class="ui-btn--accent-purple"
      > {{ t('dictionaryExport.viewCached') }} </RouterLink
    >
  </div>
   <!-- Export Form -->
  <div class="p-6 bg-white rounded-lg shadow-sm space-y-6">
     <!-- Language selection (one row, input group) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-2">
       <label class="block text-sm font-medium text-gray-700 sm:block">{{
        t('dictionaryExport.languageFrom')
      }}</label
      > <label class="block text-sm font-medium text-gray-700 sm:block">{{
        t('dictionaryExport.languageTo')
      }}</label
      >
      <div class="input-group sm:col-span-2">
         <select
          v-model="selectedSourceLanguage"
          class="input-field w-full"
          :aria-label="t('dictionaryExport.languageFrom')"
        >

          <option value=""> {{ t('dictionaryExport.sourceLangDefault') }} </option>

          <option v-for="lang in languages" :key="lang.id" :value="lang.tag">
             {{ lang.real_name }}
          </option>
           </select
        > <select
          v-model="selectedLanguage"
          class="input-field w-full"
          :aria-label="t('dictionaryExport.languageTo')"
        >

          <option value=""> {{ t('dictionaryExport.selectLanguage') }} </option>

          <option v-for="lang in languages" :key="lang.id" :value="lang.tag">
             {{ lang.real_name }}
          </option>
           </select
        >
      </div>

    </div>
     <!-- Format Selection -->
    <div class="space-y-2">
       <label class="block text-sm font-medium text-gray-700">{{
        t('dictionaryExport.formatLabel')
      }}</label
      >
      <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">

        <div
          v-for="format in exportFormats"
          :key="format.value"
          class="relative bg-white border rounded-lg cursor-pointer hover:border-blue-500 transition-colors"
          :class="{
            'border-blue-500 ring-2 ring-blue-500': selectedFormat === format.value,
            'border-gray-200': selectedFormat !== format.value,
          }"
          @click="selectedFormat = format.value"
        >

          <div class="p-4">

            <div class="flex items-center justify-between">

              <div class="flex items-center">

                <div class="flex-shrink-0 h-6 w-6" />

                <div class="ml-3">

                  <h3 class="text-sm font-medium text-gray-900">
                     {{ t(`dictionaryExport.formats.${format.value}.label`) }}
                  </h3>

                </div>

              </div>

              <div v-if="selectedFormat === format.value" class="text-blue-500">
                 <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">

                  <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clip-rule="evenodd"
                  />
                   </svg
                >
              </div>

            </div>

            <p class="mt-2 text-sm text-gray-500">
               {{ t(`dictionaryExport.formats.${format.value}.description`) }}
            </p>

          </div>

        </div>

      </div>

    </div>
     <!-- Options -->
    <div class="space-y-2">
       <label class="block text-sm font-medium text-gray-700">{{
        t('dictionaryExport.optionsLabel')
      }}</label
      >
      <div class="bg-gray-50 rounded-lg p-4 space-y-4">

        <div class="flex items-center space-x-2">
           <input
            id="positiveScoresOnly"
            v-model="positiveScoresOnly"
            type="checkbox"
            class="checkbox-toggle"
          /> <label for="positiveScoresOnly" class="block text-sm text-gray-700"
            > {{ t('dictionaryExport.positiveScoresOnly') }} </label
          >
        </div>

      </div>

    </div>
     <!-- Action Buttons -->
    <div class="flex items-center justify-end space-x-4 pt-4">

      <div v-if="isLoading" class="flex items-center text-gray-500">
         <Loader2 class="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-500" /> {{
          t('dictionaryExport.generating')
        }}
      </div>
       <button
        :disabled="!canExport || isLoading"
        class="inline-flex items-center ui-btn--read"
        @click="handleExport"
      >
         {{ t('dictionaryExport.exportButton') }} </button
      >
    </div>

  </div>

</template>

<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { getLanguages, exportDictionary } from '@/api'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()

const { showError, clearError } = useError()
const languages = ref([])
const STORAGE_KEY = 'dictionaryExport_selectedLanguage'
const SOURCE_LANG_STORAGE_KEY = 'dictionaryExport_selectedSourceLanguage'

const getInitialLanguage = (languages) => {
  if (typeof window === 'undefined') return

  // Try localStorage first
  const storedLang = localStorage.getItem(STORAGE_KEY)
  if (storedLang && languages.some((lang) => lang.tag === storedLang)) {
    return storedLang
  }
}

const getInitialSourceLanguage = (languages) => {
  if (typeof window === 'undefined') return ''

  const stored = localStorage.getItem(SOURCE_LANG_STORAGE_KEY)
  // Allow empty string (= Lojban default)
  if (stored !== null && (stored === '' || languages.some((lang) => lang.tag === stored))) {
    return stored
  }
  return ''
}

const selectedLanguage = ref('')
const selectedSourceLanguage = ref('')
const selectedFormat = ref('pdf')
const positiveScoresOnly = ref(true)
const isLoading = ref(false)

// Format options with icons
const exportFormats = [
  {
    value: 'pdf',
    label: 'PDF',
    description: 'Generate a formatted PDF document',
  },
  {
    value: 'latex',
    label: 'LaTeX',
    description: 'Export raw LaTeX source code',
  },
  {
    value: 'xml',
    label: 'XML',
    description: 'Export structured XML data',
  },
  {
    value: 'json',
    label: 'JSON',
    description: 'Export as JSON data format',
  },
  {
    value: 'tsv',
    label: 'TSV',
    description: 'Export as tab-separated values',
  },
]

// Update page title based on selected language and format
useSeoHead({
  title: computed(() => {
    const languageName =
      languages.value.find((lang) => lang.tag === selectedLanguage.value)?.real_name ||
      'Dictionary'
    const formatName =
      exportFormats.find((f) => f.value === selectedFormat.value)?.label || 'Export'
    return `${languageName} ${formatName}`
  }),
})

// Computed properties
const canExport = computed(() => {
  return selectedLanguage.value && selectedFormat.value && !isLoading.value
})

// Load languages when component mounts
onMounted(async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
    const candidateLanguage = getInitialLanguage(languages.value)
    if (candidateLanguage) selectedLanguage.value = candidateLanguage
    selectedSourceLanguage.value = getInitialSourceLanguage(languages.value)
  } catch (err) {
    showError('Failed to load languages')
  }
})

// Watch selectedLanguage and save to localStorage
watch(selectedLanguage, (newLang) => {
  if (typeof window === 'undefined') return

  if (newLang) {
    localStorage.setItem(STORAGE_KEY, newLang)
  }
})

// Watch selectedSourceLanguage and save to localStorage
watch(selectedSourceLanguage, (newLang) => {
  if (typeof window === 'undefined') return
  localStorage.setItem(SOURCE_LANG_STORAGE_KEY, newLang)
})

const handleExport = async () => {
  if (!canExport.value) return

  clearError()
  isLoading.value = true

  try {
    // Prepare params
    const paramsObj: Record<string, string> = {
      format: selectedFormat.value,
      positive_scores_only: positiveScoresOnly.value ? 'true' : 'false',
    }
    if (selectedSourceLanguage.value) {
      paramsObj.source_lang = selectedSourceLanguage.value
    }
    const params = new URLSearchParams(paramsObj).toString()

    const response = await exportDictionary(selectedLanguage.value, params)

    // Handle error response that might come as text
    if (response.status !== 200) {
      showError('Export failed')
      return
    }

    // Get filename from Content-Disposition header or generate default
    const contentDisposition = response.headers?.['content-disposition']
    const filename = contentDisposition
      ? contentDisposition.split('filename=')[1].replace(/"/g, '')
      : `dictionary-${selectedLanguage.value}.${selectedFormat.value}`

    // Create download from blob
    const url = window.URL.createObjectURL(response.data)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    window.URL.revokeObjectURL(url)
    a.remove()
  } catch (err) {
    // If it's a blob response with error message
    if (err.response?.data instanceof Blob) {
      const reader = new FileReader()
      reader.onload = () => {
        const msg = typeof reader.result === 'string' ? reader.result : ''
        showError(msg)
      }
      reader.readAsText(err.response.data)
    } else {
      showError(err.message || 'Export failed. Please try again.')
    }
  } finally {
    isLoading.value = false
  }
}
</script>

