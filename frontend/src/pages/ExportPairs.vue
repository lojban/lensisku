<template>
  <div class="max-w-4xl mx-auto p-4 sm:p-6 lg:p-8">
    <h1 class="text-3xl font-bold text-gray-800 mb-6">{{ t('exportPairs.title') }}</h1>
    
    <div class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600 mb-6">{{ t('exportPairs.description') }}</p>

      <form @submit.prevent="handleExport" class="space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- From Language -->
          <div>
            <label for="fromLang" class="block text-sm font-medium text-gray-700 mb-2">
              {{ t('exportPairs.fromLanguage') }}
            </label>
            <select
              id="fromLang"
              v-model="fromLang"
              class="input-field w-full"
              required
            >
              <option value="" disabled>{{ t('exportPairs.selectLanguage') }}</option>
              <option v-for="lang in languages" :key="lang.id" :value="lang.id">
                {{ lang.real_name }} ({{ lang.english_name }})
              </option>
            </select>
          </div>

          <!-- To Language -->
          <div>
            <label for="toLang" class="block text-sm font-medium text-gray-700 mb-2">
              {{ t('exportPairs.toLanguage') }}
            </label>
            <select
              id="toLang"
              v-model="toLang"
              class="input-field w-full"
              required
            >
              <option value="" disabled>{{ t('exportPairs.selectLanguage') }}</option>
              <option v-for="lang in languages" :key="lang.id" :value="lang.id">
                {{ lang.real_name }} ({{ lang.english_name }})
              </option>
            </select>
          </div>
        </div>

        <div class="pt-4">
          <button
            type="submit"
            class="btn-aqua-emerald w-full sm:w-auto flex items-center justify-center gap-2"
            :disabled="isExporting || !isValid"
          >
            <Download v-if="!isExporting" class="h-5 w-5" />
            <Loader2 v-else class="h-5 w-5 animate-spin" />
            <span>{{ isExporting ? t('exportPairs.exporting') : t('exportPairs.exportButton') }}</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Download, Loader2 } from 'lucide-vue-next'
import { getLanguages, exportLinkedPairs } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'
import { useError } from '@/composables/useError'

const { t, locale } = useI18n()
const { showError } = useError()

useSeoHead({
  title: t('exportPairs.pageTitle'),
  description: t('exportPairs.pageDescription')
})

const languages = ref([])
const fromLang = ref(1) // Default Lojban
const toLang = ref(2)   // Default English
const isExporting = ref(false)

const isValid = computed(() => fromLang.value && toLang.value && fromLang.value !== toLang.value)

onMounted(async () => {
  try {
    const res = await getLanguages()
    languages.value = res.data
  } catch (e) {
    console.error('Failed to load languages', e)
    showError(t('exportPairs.loadError'))
  }
})

const handleExport = async () => {
  if (!isValid.value) return

  isExporting.value = true
  try {
    const response = await exportLinkedPairs(fromLang.value, toLang.value)
    
    // Create blob link to download
    const url = window.URL.createObjectURL(new Blob([response.data]))
    const link = document.createElement('a')
    link.href = url
    
    // Extract filename from header or generate default
    const contentDisposition = response.headers['content-disposition']
    let fileName = `pairs_${fromLang.value}_${toLang.value}.tsv`
    if (contentDisposition) {
      const fileNameMatch = contentDisposition.match(/filename="?(.+)"?/)
      if (fileNameMatch && fileNameMatch.length === 2) fileName = fileNameMatch[1]
    }
    
    link.setAttribute('download', fileName)
    document.body.appendChild(link)
    link.click()
    link.remove()
  } catch (e) {
    console.error('Export failed', e)
    showError(t('exportPairs.exportError'))
  } finally {
    isExporting.value = false
  }
}
</script>
