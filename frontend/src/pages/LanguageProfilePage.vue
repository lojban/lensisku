<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex flex-col lg:flex-row justify-between items-center gap-4 mb-6">
      <h2 class="text-2xl font-bold text-gray-800 text-center sm:text-left flex-1 min-w-[200px]">
        {{ language ? language.english_name : t('languageList.title') }}
      </h2>

      <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-center items-center">
        <RouterLink
          v-if="language"
          :to="`/?mode=dictionary&langs=${language.id}`"
          class="ui-btn--read ui-btn--group-item"
        >
          {{ t('languageList.viewInDictionary') }}
        </RouterLink>

        <a
          v-if="language?.url"
          :href="language.url"
          target="_blank"
          rel="noopener noreferrer"
          class="ui-btn--market ui-btn--group-item"
        >
          {{ t('languageList.visitWebsite') }}
        </a>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="flex flex-col items-center justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      <p class="mt-2 text-gray-600">{{ t('languageList.loading') }}</p>
    </div>

    <!-- Details -->
    <div v-else-if="language" class="bg-white p-4 rounded-lg border space-y-4">
      <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-3">
        <div>
          <p class="text-sm font-medium text-gray-500">
            {{ t('languageList.tableHeader.tag') }}:
          </p>
          <p class="text-gray-900 mt-1">{{ language.tag }}</p>

          <p class="text-gray-600 mt-3">
            {{ language.lojban_name }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="space-y-4">
          <div>
            <span class="text-sm font-medium text-gray-500">{{ t('languageList.tableHeader.englishName') }}</span>
            <p class="mt-1 text-gray-900">{{ language.english_name }}</p>
          </div>

          <div>
            <span class="text-sm font-medium text-gray-500">{{ t('languageList.tableHeader.lojbanName') }}</span>
            <p class="mt-1 text-gray-900">{{ language.lojban_name }}</p>
          </div>

          <div>
            <span class="text-sm font-medium text-gray-500">{{ t('languageList.tableHeader.realName') }}</span>
            <p class="mt-1 text-gray-900">{{ language.real_name }}</p>
          </div>
        </div>

        <div class="space-y-4">
          <div>
            <span class="text-sm font-medium text-gray-500">{{ t('languageList.tableHeader.forLojban') }}</span>
            <p class="mt-1 text-gray-900">
              {{ language.for_lojban ? t('languageList.filters.forLojban.yes') : t('languageList.filters.forLojban.no') }}
            </p>
          </div>

          <div>
            <span class="text-sm font-medium text-gray-500">{{ t('languageList.tableHeader.url') }}</span>
            <p class="mt-1">
              <a
                v-if="language.url"
                :href="language.url"
                target="_blank"
                rel="noopener noreferrer"
                class="text-blue-600 hover:underline"
              >
                {{ language.url }}
              </a>
              <span v-else class="text-gray-900">{{ t('profile.notSet') }}</span>
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Not found -->
    <div v-else class="bg-gray-50 border border-gray-200 rounded-xl p-8 text-center text-gray-600">
      {{ t('languageList.noLanguages') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, RouterLink } from 'vue-router'
import { getLanguages } from '@/api'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

type LanguageRow = {
  id: number
  tag: string
  english_name: string
  lojban_name: string
  real_name: string
  for_lojban?: string | null
  url?: string | null
}

const route = useRoute()
const { t } = useI18n()
const { showError, clearError } = useError()

const isLoading = ref(true)
const language = ref<LanguageRow | null>(null)

const languageId = computed(() => {
  const raw = route.params.id
  const s = Array.isArray(raw) ? raw[0] : raw
  const n = parseInt(String(s), 10)
  return Number.isFinite(n) ? n : null
})

const fetchLanguage = async () => {
  if (languageId.value === null) {
    language.value = null
    isLoading.value = false
    return
  }

  isLoading.value = true
  clearError()

  try {
    const res = await getLanguages()
    language.value = (res.data as LanguageRow[]).find((l) => l.id === languageId.value) ?? null
  } catch (e: unknown) {
    const maybeAxiosError = e as { response?: { data?: { error?: string } } }
    showError(maybeAxiosError.response?.data?.error || t('languageList.loadError'))
    language.value = null
  } finally {
    isLoading.value = false
  }
}

useSeoHead({
  title: computed(() => language.value?.english_name || ''),
  robots: 'noindex, nofollow',
})

watch(languageId, () => fetchLanguage(), { immediate: true })
</script>

