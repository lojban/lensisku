<template>
  <div class="bg-white rounded-lg shadow-sm p-6">
    <!-- Header -->
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800">
        {{ t('versionHistory.title') }}
      </h2>
      <RouterLink
        :to="`/valsi/${valsiId}`"
        class="btn-history"
      >
        <ArrowLeft class="h-5 w-5" />
      </RouterLink>
    </div>

    <!-- Loading State -->
    <div
      v-if="isLoading"
      class="flex justify-center py-8"
    >
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>

    <!-- Version List -->
    <div
      v-else
      class="space-y-4"
    >
      <div
        v-for="version in versions"
        :key="version.version_id"
        class="border rounded-lg p-4 hover:border-blue-300 transition-colors"
      >
        <div class="flex justify-between items-start mb-2">
          <div class="space-x-2">
            <span class="font-medium text-gray-800">{{ t('versionHistory.versionBy', { versionId: version.version_id, username: version.username }) }}</span>
            <p
              v-if="version.commit_message"
              class="text-sm text-gray-600 mt-1 italic"
            >
              {{ t('versionHistory.commitMessage', { message: version.commit_message }) }}
            </p>
          </div>
          <span class="text-sm text-gray-500">
            {{ formatDate(version.created_at) }}
          </span>
        </div>

        <!-- Version Details -->
        <div class="space-y-2 text-sm text-gray-600">
          <div>
            <span class="font-medium">{{ t('versionHistory.definitionLabel') }}</span>
            <div class="mt-1 bg-gray-50 p-2 rounded">
              <LazyMathJax :content="version.content.definition" />
            </div>
          </div>

          <div v-if="version.content.notes">
            <span class="font-medium">{{ t('versionHistory.notesLabel') }}</span>
            <div class="mt-1 bg-gray-50 p-2 rounded">
              <LazyMathJax :content="version.content.notes" />
            </div>
          </div>

          <!-- Keywords -->
          <div
            v-if="version.content.gloss_keywords?.length"
            class="mt-2"
          >
            <span class="font-medium">{{ t('versionHistory.glossKeywordsLabel') }}</span>
            <div class="flex flex-wrap gap-2 mt-1">
              <span
                v-for="(keyword, idx) in version.content.gloss_keywords"
                :key="idx"
                class="inline-flex items-center px-2 py-1 rounded-md text-sm font-medium bg-blue-100 text-blue-800"
              >
                {{ keyword.word }}
                <span
                  v-if="keyword.meaning"
                  class="ml-1 text-blue-600"
                >({{ keyword.meaning }})</span>
              </span>
            </div>
          </div>

          <div
            v-if="version.content.place_keywords?.length"
            class="mt-2"
          >
            <span class="font-medium">{{ t('versionHistory.placeKeywordsLabel') }}</span>
            <div class="flex flex-wrap gap-2 mt-1">
              <span
                v-for="(keyword, idx) in version.content.place_keywords"
                :key="idx"
                class="inline-flex items-center px-2 py-1 rounded-md text-sm font-medium bg-green-100 text-green-800"
              >
                {{ keyword.word }}
                <span
                  v-if="keyword.meaning"
                  class="ml-1 text-green-600"
                >({{ keyword.meaning }})</span>
              </span>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="mt-4 flex items-center space-x-4">
          <button
            v-if="selectedVersion && selectedVersion !== version.version_id"
            class="px-3 py-1 text-sm bg-blue-50 text-blue-600 rounded hover:bg-blue-100"
            @click="compareVersions(version.version_id)"
          >
            {{ t('versionHistory.compareWith', { versionId: selectedVersion }) }}
          </button>
          <button
            v-else-if="!selectedVersion"
            class="btn-get"
            @click="selectedVersion = version.version_id"
          >
            {{ t('versionHistory.compareThis') }}
          </button>
          <button
            v-if="selectedVersion === version.version_id"
            class="btn-cancel"
            @click="selectedVersion = null"
          >
            {{ t('versionHistory.cancelSelection') }}
          </button>
          <button
            v-if="auth.state.isLoggedIn"
            class="btn-revert"
            @click="performRevertToVersion(version.version_id)"
          >
            {{ t('versionHistory.revertToThis') }}
          </button>
        </div>
      </div>
    </div>

    <PaginationComponent
      v-if="!isLoading && totalPages > 1"
      :current-page="currentPage"
      :total-pages="totalPages"
      :total="total"
      :per-page="10"
      class="mt-6"
      @prev="() => changePage(currentPage - 1)"
      @next="() => changePage(currentPage + 1)"
    />
  </div>

  <!-- Version Comparison ModalComponent -->
  <ModalComponent
    :show="showDiffModal"
    :title="t('versionHistory.comparisonTitle')"
    @close="closeDiffModal"
  >
    <div class="max-w-4xl">
      <div
        v-if="versionDiff"
        class="space-y-4"
      >
        <div
          v-for="change in versionDiff.changes"
          :key="change.field"
          class="border rounded-lg p-4"
        >
          <h4 class="font-medium text-gray-700 mb-2">
            {{ formatFieldName(change.field) }}
          </h4>

          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <div class="text-sm text-gray-500">
                {{ t('versionHistory.oldVersion') }}
              </div>
              <div
                v-if="change.old_value"
                class="bg-red-50 p-2 rounded text-red-700"
              >
                <LazyMathJax :content="change.old_value" />
              </div>
              <div
                v-else
                class="text-gray-400 italic"
              >
                {{ t('versionHistory.noContent') }}
              </div>
            </div>

            <div class="space-y-2">
              <div class="text-sm text-gray-500">
                {{ t('versionHistory.newVersion') }}
              </div>
              <div
                v-if="change.new_value"
                class="bg-green-50 p-2 rounded text-green-700"
              >
                <LazyMathJax :content="change.new_value" />
              </div>
              <div
                v-else
                class="text-gray-400 italic"
              >
                {{ t('versionHistory.noContent') }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </ModalComponent>
</template>

<script setup>
import { ArrowLeft } from 'lucide-vue-next'
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import { getVersionHistory, getVersionDiff, revertToVersion } from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()

const route = useRoute()
const auth = useAuth()

const isLoading = ref(true)
const { showError, clearError } = useError()
const versions = ref([])
const currentPage = ref(1)
const totalPages = ref(1)
const total = ref(0)
const selectedVersion = ref(null)
const showDiffModal = ref(false)
const versionDiff = ref(null)
const valsiId = ref(route.query.valsi_id)

const props = defineProps({
  id: {
    type: String,
    default: '0',
  },
})

// Fetch version history
const fetchVersionHistory = async (page = 1) => {
  isLoading.value = true
  clearError()

  try {
    const response = await getVersionHistory(props.id, {
      page,
      per_page: 10,
    })

    const data = response.data
    versions.value = data.versions
    total.value = data.total
    totalPages.value = Math.ceil(data.total / data.per_page)
    currentPage.value = page
  } catch (e) {
    showError(t('versionHistory.loadError'))
    console.error('Error loading versions:', e)
  } finally {
    isLoading.value = false
  }
}

// Compare versions
const compareVersions = async (toVersion) => {
  try {
    const response = await getVersionDiff(toVersion, selectedVersion.value)
    versionDiff.value = response.data
    showDiffModal.value = true
  } catch (e) {
    showError(t('versionHistory.compareError'))
    console.error('Error comparing versions:', e)
  }
}

// Revert to version
const performRevertToVersion = async (versionId) => {
  if (!confirm(t('versionHistory.revertConfirm'))) {
    return
  }

  try {
    await revertToVersion(versionId)
    await fetchVersionHistory(currentPage.value)
  } catch (e) {
    showError(t('versionHistory.revertError'))
    console.error('Error reverting:', e)
  }
}

// Helper functions
const formatDate = (date) => {
  return new Date(date).toLocaleString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const formatFieldName = (field) => {
  return field
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

const changePage = (page) => {
  fetchVersionHistory(page)
}

const closeDiffModal = () => {
  showDiffModal.value = false
  versionDiff.value = null
  selectedVersion.value = null
}

// Initialize SEO metadata using i18n translations
const pageTitle = computed(() => t('versionHistory.title'))
const metaDescription = computed(() => t('versionHistory.metaDescription'))

useSeoHead({
  title: pageTitle,
  meta: [
    {
      name: 'description',
      content: metaDescription
    }
  ]
}, locale.value)

onMounted(() => {
  fetchVersionHistory()
})
</script>
