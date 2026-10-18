<template>
  <div class="space-y-4">
    <!-- Header with stats -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 v-if="!embedded" class="text-2xl font-bold text-gray-800">
          {{ t('languageList.title') }}
        </h2>
        <p class="text-gray-600" :class="embedded ? '' : 'mt-2'">
          {{ t('languageList.description') }}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2 shrink-0">
        <Button v-if="canManage" variant="create" type="button" @click="openCreate">
          {{ t('languageList.addLanguage') }}
        </Button>
        <div
          v-if="languages.length > 0"
          class="text-sm text-gray-600 bg-gray-100 px-3 py-1.5 rounded-full font-medium shrink-0"
        >
          {{
            hasActiveSearchOrFilters
              ? t('languageList.showingCount', {
                  count: filteredLanguages.length,
                  total: languages.length,
                })
              : t('languageList.totalLabel', { count: languages.length })
          }}
        </div>
      </div>
    </div>

    <Teleport defer to="#iam-tab-toolbar" :disabled="!embedded">
      <div v-if="!isLoading && languages.length > 0" class="toolbar-panel">
        <div class="toolbar-row">
          <div class="toolbar-search-slot">
            <SearchInput
              v-model="searchQuery"
              :is-loading="isSearching"
              :placeholder="t('languageList.searchPlaceholder')"
              :show-search-icon="true"
              @clear="clearSearch"
            />
          </div>

          <div class="toolbar-field-row">
            <label class="toolbar-control-label">{{
              t('languageList.filters.forLojbanLabel')
            }}</label>
            <div class="toolbar-dropdown-anchor">
              <ToolbarSelectDropdown>
                <template #label>{{ forLojbanTriggerLabel }}</template>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': !forLojbanFilter }"
                  @click="setForLojbanFilter('')"
                >
                  {{ t('languageList.filters.all') }}
                </ToolbarSelectDropdownItem>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': forLojbanFilter === 'yes' }"
                  @click="setForLojbanFilter('yes')"
                >
                  {{ t('languageList.filters.forLojban.yes') }}
                </ToolbarSelectDropdownItem>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': forLojbanFilter === 'no' }"
                  @click="setForLojbanFilter('no')"
                >
                  {{ t('languageList.filters.forLojban.no') }}
                </ToolbarSelectDropdownItem>
              </ToolbarSelectDropdown>
            </div>
          </div>

          <div class="toolbar-field-row">
            <label class="toolbar-control-label">{{ t('languageList.filters.urlLabel') }}</label>
            <div class="toolbar-dropdown-anchor">
              <ToolbarSelectDropdown>
                <template #label>{{ urlTriggerLabel }}</template>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': !urlFilter }"
                  @click="setUrlFilter('')"
                >
                  {{ t('languageList.filters.all') }}
                </ToolbarSelectDropdownItem>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': urlFilter === 'yes' }"
                  @click="setUrlFilter('yes')"
                >
                  {{ t('languageList.filters.url.hasUrl') }}
                </ToolbarSelectDropdownItem>
                <ToolbarSelectDropdownItem
                  :class="{ 'bg-gray-100': urlFilter === 'no' }"
                  @click="setUrlFilter('no')"
                >
                  {{ t('languageList.filters.url.noUrl') }}
                </ToolbarSelectDropdownItem>
              </ToolbarSelectDropdown>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex flex-col items-center justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      <p class="mt-2 text-gray-600">{{ t('languageList.loading') }}</p>
    </div>

    <!-- Main content -->
    <div v-else class="space-y-3">
      <!-- Empty state: no languages -->
      <div
        v-if="languages.length === 0"
        class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200"
      >
        <p class="text-gray-600">{{ t('languageList.noLanguages') }}</p>
      </div>

      <!-- Empty state: no matches -->
      <div
        v-else-if="filteredLanguages.length === 0"
        class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200"
      >
        <p class="text-gray-600">{{ t('languageList.noMatchingLanguages') }}</p>
        <Button variant="read" type="button" class="mt-3" @click="clearSearchAndFilters">
          {{ t('languageList.clearSearch') }}
        </Button>
      </div>

      <div v-else class="grid gap-3 sm:gap-4">
        <ListRowSurface
          v-for="lang in filteredLanguages"
          :key="lang.id"
          @click="openLanguage(lang)"
        >
          <div class="flex items-start gap-4">
            <div class="shrink-0 mt-1">
              <div
                class="avatar-placeholder-sm overflow-hidden px-1 text-center text-[10px] font-semibold leading-tight tracking-tight text-gray-600 [overflow-wrap:anywhere]"
                :title="lang.tag"
              >
                {{ lang.tag }}
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex justify-between items-start gap-2 min-w-0">
                <div class="min-w-0 flex-1 pr-1">
                  <h3 class="text-lg font-medium text-blue-600 break-words hover:text-blue-700">
                    {{ lang.english_name }}
                  </h3>
                  <p class="text-gray-600 text-sm mt-0.5 break-words">{{ lang.lojban_name }}</p>
                </div>

                <span
                  class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium shrink-0"
                  :class="
                    lang.for_lojban ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                  "
                >
                  {{ t('languageList.tableHeader.forLojban') }}:
                  {{
                    lang.for_lojban
                      ? t('languageList.filters.forLojban.yes')
                      : t('languageList.filters.forLojban.no')
                  }}
                </span>
              </div>

              <div class="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-400">
                <span>{{ lang.real_name }}</span>
                <span>{{ lang.tag }}</span>
                <RouterLink
                  :to="`/?mode=dictionary&langs=${lang.id}`"
                  class="text-blue-600 hover:underline"
                  @click.stop
                >
                  {{ t('languageList.viewInDictionary') }}
                </RouterLink>
                <a
                  v-if="lang.url"
                  :href="lang.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-blue-600 hover:underline"
                  @click.stop
                >
                  {{ t('languageList.visitWebsite') }}
                </a>
                <button
                  v-if="canManage"
                  type="button"
                  class="text-blue-600 hover:underline"
                  @click.stop="openEdit(lang)"
                >
                  {{ t('languageList.editLanguage') }}
                </button>
              </div>
            </div>
          </div>
        </ListRowSurface>
      </div>
    </div>

    <ModalComponent
      :show="showEditor"
      :title="editingId ? t('languageList.editLanguage') : t('languageList.addLanguage')"
      @close="closeEditor"
    >
      <form class="space-y-3" @submit.prevent="saveLanguage">
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.tag') }}
          <input v-model="draft.tag" class="input-field mt-1 w-full" required maxlength="128" />
        </label>
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.englishName') }}
          <input v-model="draft.english_name" class="input-field mt-1 w-full" required />
        </label>
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.lojbanName') }}
          <input v-model="draft.lojban_name" class="input-field mt-1 w-full" required />
        </label>
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.realName') }}
          <input v-model="draft.real_name" class="input-field mt-1 w-full" required />
        </label>
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.forLojban') }}
          <input
            v-model="draft.for_lojban"
            class="input-field mt-1 w-full"
            :placeholder="t('languageList.forLojbanPlaceholder')"
          />
        </label>
        <label class="block text-sm font-medium text-gray-700">
          {{ t('languageList.tableHeader.url') }}
          <input
            v-model="draft.url"
            class="input-field mt-1 w-full"
            :placeholder="t('languageList.urlPlaceholder')"
          />
        </label>
        <p v-if="editorError" class="form-callout--error">{{ editorError }}</p>
      </form>
      <template #footer>
        <div class="flex justify-end gap-2">
          <Button variant="neutral" type="button" @click="closeEditor">
            {{ t('modal.close') }}
          </Button>
          <Button variant="edit" type="button" :disabled="isSaving" @click="saveLanguage">
            {{ t('languageList.save') }}
          </Button>
        </div>
      </template>
    </ModalComponent>
  </div>
</template>

<script setup lang="ts">
import {
  Button,
  ListRowSurface,
  ToolbarSelectDropdown,
  ToolbarSelectDropdownItem,
} from '@packages/ui'
import { ref, computed, onMounted, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import SearchInput from '@/components/SearchInput.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { createLanguage, getLanguages, updateLanguage } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  embedded: { type: Boolean, default: false },
})

const { t } = useI18n()
const router = useRouter()
const auth = useAuth()
const canManage = computed(() => auth.state.authorities?.includes('manage_languages') ?? false)

type LanguageRow = {
  id: number
  tag: string
  english_name: string
  lojban_name: string
  real_name: string
  for_lojban?: string | null
  url?: string | null
}

const languages = ref<LanguageRow[]>([])
const searchQuery = ref('')
const isLoading = ref(true)
const isSearching = ref(false)
const { showError, clearError } = useError()

const forLojbanFilter = ref<'' | 'yes' | 'no'>('')
const urlFilter = ref<'' | 'yes' | 'no'>('')

const hasActiveSearchOrFilters = computed(() => {
  const qActive = (searchQuery.value || '').trim().length > 0
  return qActive || !!forLojbanFilter.value || !!urlFilter.value
})

const forLojbanTriggerLabel = computed(() => {
  if (forLojbanFilter.value === 'yes') return t('languageList.filters.forLojban.yes')
  if (forLojbanFilter.value === 'no') return t('languageList.filters.forLojban.no')
  return t('languageList.filters.all')
})

const urlTriggerLabel = computed(() => {
  if (urlFilter.value === 'yes') return t('languageList.filters.url.hasUrl')
  if (urlFilter.value === 'no') return t('languageList.filters.url.noUrl')
  return t('languageList.filters.all')
})

const filteredLanguages = computed(() => {
  const q = (searchQuery.value || '').trim().toLowerCase()

  const matchesSearch = (lang: LanguageRow) => {
    if (!q) return true
    return (
      (lang.tag && lang.tag.toLowerCase().includes(q)) ||
      (lang.english_name && lang.english_name.toLowerCase().includes(q)) ||
      (lang.lojban_name && lang.lojban_name.toLowerCase().includes(q)) ||
      (lang.real_name && lang.real_name.toLowerCase().includes(q))
    )
  }

  const matchesForLojban = (lang: LanguageRow) => {
    if (!forLojbanFilter.value) return true
    const isYes = !!lang.for_lojban
    return forLojbanFilter.value === 'yes' ? isYes : !isYes
  }

  const matchesUrl = (lang: LanguageRow) => {
    if (!urlFilter.value) return true
    const has = !!(lang.url && lang.url.trim())
    return urlFilter.value === 'yes' ? has : !has
  }

  return languages.value.filter(
    (lang) => matchesSearch(lang) && matchesForLojban(lang) && matchesUrl(lang)
  )
})

const setForLojbanFilter = (v: '' | 'yes' | 'no') => {
  forLojbanFilter.value = v
}

const setUrlFilter = (v: '' | 'yes' | 'no') => {
  urlFilter.value = v
}

const clearSearch = () => {
  searchQuery.value = ''
  isSearching.value = false
}

const clearSearchAndFilters = () => {
  searchQuery.value = ''
  forLojbanFilter.value = ''
  urlFilter.value = ''
  isSearching.value = false
}

const openLanguage = (lang: LanguageRow) => {
  router.push(`/language/${lang.id}`)
}

const emptyDraft = () => ({
  tag: '',
  english_name: '',
  lojban_name: '',
  real_name: '',
  for_lojban: '',
  url: '',
})

const showEditor = ref(false)
const editingId = ref<number | null>(null)
const draft = ref(emptyDraft())
const editorError = ref('')
const isSaving = ref(false)

const openCreate = () => {
  editingId.value = null
  draft.value = emptyDraft()
  editorError.value = ''
  showEditor.value = true
}

const openEdit = (lang: LanguageRow) => {
  editingId.value = lang.id
  draft.value = {
    tag: lang.tag,
    english_name: lang.english_name,
    lojban_name: lang.lojban_name,
    real_name: lang.real_name,
    for_lojban: lang.for_lojban || '',
    url: lang.url || '',
  }
  editorError.value = ''
  showEditor.value = true
}

const closeEditor = () => {
  showEditor.value = false
}

const saveLanguage = async () => {
  editorError.value = ''
  isSaving.value = true
  const body = {
    tag: draft.value.tag.trim(),
    english_name: draft.value.english_name.trim(),
    lojban_name: draft.value.lojban_name.trim(),
    real_name: draft.value.real_name.trim(),
    for_lojban: draft.value.for_lojban.trim() || null,
    url: draft.value.url.trim() || null,
  }
  try {
    const response = editingId.value
      ? await updateLanguage(editingId.value, body)
      : await createLanguage(body)
    const saved = response.data as LanguageRow
    languages.value = [...languages.value.filter((lang) => lang.id !== saved.id), saved].sort(
      (a, b) => a.real_name.localeCompare(b.real_name)
    )
    showEditor.value = false
  } catch (e) {
    editorError.value = e.response?.data?.error || t('languageList.saveError')
  } finally {
    isSaving.value = false
  }
}

const fetchLanguages = async () => {
  isLoading.value = true
  clearError()

  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (e) {
    showError(e.response?.data?.error || t('languageList.loadError'))
    console.error('Error fetching languages:', e)
  } finally {
    isLoading.value = false
  }
}

const pageTitle = ref(t('languageList.title'))

onMounted(() => {
  fetchLanguages()
})

watch(languages, (newLanguages) => {
  if (newLanguages.length > 0) {
    pageTitle.value = `${t('languageList.title')} (${newLanguages.length})`
  } else {
    pageTitle.value = t('languageList.title')
  }
})

if (!props.embedded) {
  useSeoHead({ title: pageTitle, pathWithoutLocale: '/languages' })
}
</script>
