<template>
  <div class="flex flex-col gap-4">
    <StudyStreak />

    <PageHeader title-as="h2" title-tone="secondary" margin="none" class="mt-0">
      <template #title>
        <span class="select-none">
          {{
            viewMode !== 'my'
              ? t('collectionList.publicCollections')
              : t('collectionList.myCollections')
          }}
        </span>
      </template>
      <template #description>
        <div class="flex max-w-2xl flex-col gap-2 text-sm text-slate-600">
          <template v-if="auth.state.isLoggedIn">
            <p class="m-0">{{ t('collectionList.collectionDescriptionIntro') }}</p>
            <p class="m-0">{{ t('collectionList.collectionDescriptionCourse') }}</p>
          </template>
          <p v-else class="m-0">
            <i18n-t keypath="collectionList.collectionDescriptionLoggedOut" tag="span">
              <RouterLink
                to="/login"
                class="font-medium text-blue-600 underline hover:text-blue-800"
                >{{ t('collectionList.loginTo') }}</RouterLink
              >
            </i18n-t>
          </p>
        </div>
      </template>
      <template #trailing>
        <div
          v-if="auth.state.isLoggedIn"
          class="flex min-w-0 flex-grow flex-row flex-wrap justify-end gap-2"
        >
          <FileInput
            ref="importFileInput"
            type="file"
            accept=".json"
            class="hidden"
            @change="handleImportFile"
          />
          <div class="flex flex-row items-center gap-2">
            <IconButton
              v-if="viewMode === 'my'"
              :label="t('collectionList.publicCollectionsLabel')"
              button-classes="ui-btn--neutral [&>span>span]:hidden sm:[&>span>span]:inline"
              @click="setViewMode('public')"
            >
              <template #icon> <ArrowBigRight class="h-4 w-4" /> </template>
            </IconButton>
            <ToolbarSelectDropdown>
              <template #label>{{ t('collectionList.addActions') }}</template>
              <ToolbarSelectDropdownItem
                class="text-orange-600 hover:bg-orange-50"
                @click="setViewMode('my')"
              >
                <BookOpen class="h-4 w-4 shrink-0" /> {{ t('collectionList.myCollectionsLabel') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                class="text-cyan-600 hover:bg-cyan-50"
                :disabled="isImporting"
                @click="triggerImport"
              >
                <Import class="h-4 w-4 shrink-0" /> {{ t('collectionList.importCollection') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                class="text-emerald-600 hover:bg-emerald-50"
                @click="showCreateModal = true"
              >
                <CirclePlus class="h-4 w-4 shrink-0" /> {{ t('collectionList.createCollection') }}
              </ToolbarSelectDropdownItem>
            </ToolbarSelectDropdown>
          </div>
        </div>
      </template>
    </PageHeader>
    <!-- Sort & filter controls (card-base, input-field, checkbox-toggle per brandbook). overflow-visible so sort glow/drop-shadow is not clipped by card-base overflow-hidden. -->

    <div class="card-base card-compact p-4 sm:p-5 flex flex-col gap-4 overflow-visible">
      <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
        <Input
          v-model="searchQuery"
          type="text"
          class="input-field w-full min-w-0 sm:flex-1 sm:max-w-md"
          :placeholder="
            hasFlashcardsOnly
              ? t('collectionList.searchPlaceholder')
              : t('collectionList.searchPlaceholderCollections')
          "
        />
        <label
          class="inline-flex items-center gap-2 self-end text-sm text-gray-700 select-none cursor-pointer sm:self-auto"
        >
          <Checkbox v-model="hasFlashcardsOnly" class="checkbox-toggle" />
          <span>{{ t('collectionList.onlyWithFlashcards') }}</span>
        </label>
      </div>

      <div class="flex flex-row items-center gap-2 sm:block">
        <span id="collection-list-sort-legend" class="sr-only">{{ t('sort.sortByLabel') }}</span>
        <div
          class="btn-group-forced flex flex-nowrap justify-center min-w-0 overflow-visible py-2"
          role="group"
          aria-labelledby="collection-list-sort-legend"
          aria-describedby="collection-list-sort-current"
        >
          <Button
            v-for="opt in sortOptions"
            :key="opt.value"
            variant="empty"
            type="button"
            class="ui-btn--group-item relative flex h-6 shrink-0 items-center justify-center gap-1.5 px-2 sm:px-4 !cursor-pointer"
            :class="[sortBy === opt.value ? opt.aquaClass : 'ui-btn--empty']"
            :title="opt.label"
            :aria-label="opt.label"
            :aria-pressed="sortBy === opt.value"
            @click="sortBy = opt.value"
          >
            <component
              :is="opt.icon"
              class="h-4 w-4 shrink-0 transition-[opacity,filter] duration-200"
              :class="
                sortBy === opt.value
                  ? 'opacity-100 drop-shadow-[0_0_1px_rgba(30,64,175,0.9)]'
                  : 'opacity-55'
              "
              aria-hidden="true"
            /><span class="hidden sm:inline">{{ opt.label }}</span>
          </Button>
        </div>
        <span
          id="collection-list-sort-current"
          class="min-w-0 shrink-0 text-sm text-gray-700 sm:hidden"
          aria-live="polite"
          >{{ selectedSortLabel }}</span
        >
      </div>
    </div>
    <!-- Loading State -->
    <LoadingSpinner v-if="isLoading" />
    <!-- Collections Card Grid -->
    <div v-else class="collections-section">
      <div class="collections-grid">
        <CollectionCard
          v-for="collection in collections"
          :key="collection.collection_id"
          :collection="collection"
          :cover-image-url="
            collection.has_cover_image
              ? getCollectionImage(collection.collection_id, { cached: true })
              : null
          "
          :study-loading="studyLoadingId === collection.collection_id"
          :format-date="formatDate"
          :study-button-label="t('collectionList.studyButton')"
          :collection-button-label="t('collectionList.collectionButton')"
          :flashcards-button-label="t('collectionList.flashcardsButton')"
          :discussion-button-label="t('collectionList.discussionButton')"
          :created-by-label="t('collectionList.createdBy')"
          :updated-label="t('collectionList.updatedAt')"
          :public-label="t('collectionList.publicStatus')"
          :private-label="t('collectionList.privateStatus')"
          :items-count-label="t('collectionList.itemsCount', { count: collection.item_count })"
          @study="startStudy(collection)"
        />
      </div>
      <PaginationComponent
        v-if="totalPages > 1"
        :current-page="currentPage"
        :total-pages="totalPages"
        :total="totalCollections"
        :per-page="perPage"
        class="w-full"
        @prev="prevPage"
        @next="nextPage"
      />
    </div>
    <!-- Empty State -->
    <EmptyStatePanel v-if="!isLoading && collections.length === 0">
      <Button
        v-if="viewMode === 'my' && auth.state.isLoggedIn"
        variant="create"
        class="mt-4"
        @click="showCreateModal = true"
      >
        <CirclePlus class="h-4 w-4" /> <span>{{ t('collectionList.createFirstCollection') }}</span>
      </Button>
    </EmptyStatePanel>
  </div>
  <!-- Create Collection ModalComponent -->
  <div
    v-if="showCreateModal"
    class="z-[1000] fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4"
  >
    <div class="bg-white rounded-lg max-w-md w-full p-6">
      <h3 class="text-lg font-semibold mb-4">{{ t('collectionList.createModalTitle') }}</h3>

      <form @submit.prevent="performCreateCollection">
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{
              t('collectionList.nameLabel')
            }}</label>
            <Input v-model="newCollection.name" type="text" required class="w-full input-field" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{
              t('collectionList.descriptionLabel')
            }}</label>
            <Textarea v-model="newCollection.description" rows="3" class="textarea-field" />
          </div>

          <div class="flex items-center gap-2">
            <Checkbox id="is_public" v-model="newCollection.is_public" class="checkbox-toggle" />
            <label for="is_public" class="text-sm text-gray-700">
              {{ t('collectionList.makePublicLabel') }}
            </label>
          </div>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <Button variant="cancel" type="button" @click="showCreateModal = false">
            {{ t('collectionList.cancelButton') }}
          </Button>
          <Button variant="create" type="submit" :disabled="isSubmitting">
            {{
              isSubmitting ? t('collectionList.creatingButton') : t('collectionList.createButton')
            }}
          </Button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  CirclePlus,
  Import,
  BookOpen,
  ArrowBigRight,
  CalendarDays,
  Calendar,
  Trophy,
  ArrowDown,
} from 'lucide-vue-next'
import { ref, computed, onBeforeUnmount, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import {
  getCollections,
  getPublicCollections,
  createCollection,
  importCollectionFull,
  getLevels,
  getCollectionImage,
} from '@/api'
import {
  Button,
  Checkbox,
  CollectionCard,
  EmptyStatePanel,
  FileInput,
  IconButton,
  Input,
  Textarea,
  ToolbarSelectDropdown,
  ToolbarSelectDropdownItem,
} from '@packages/ui'
import PageHeader from '@/components/layout/PageHeader.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import StudyStreak from '@/components/StudyStreak.vue'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'

const auth = useAuth()
const router = useRouter()
const route = useRoute()

const VIEW_PARAM = 'view'
const VIEW_STORAGE_KEY = 'collections-view'
const validView = (v) => (v === 'my' || v === 'public' ? v : null)

const { t, locale } = useI18n()

// State: viewMode from URL param; when no param, from localStorage if set; else default by login
const initialView = () => {
  const fromUrl = validView(route.query[VIEW_PARAM])
  if (fromUrl && (fromUrl === 'public' || auth.state.isLoggedIn)) return fromUrl
  const fromStorage = validView(
    typeof localStorage !== 'undefined' ? localStorage.getItem(VIEW_STORAGE_KEY) : null
  )
  if (fromStorage != null && (fromStorage === 'public' || auth.state.isLoggedIn)) return fromStorage
  return auth.state.isLoggedIn ? 'my' : 'public'
}
const viewMode = ref(initialView())
const collections = ref([])
/** Full-page spinner only before the first completed list fetch; refetches keep prior rows visible. */
const isLoading = ref(true)
const hasLoadedOnce = ref(false)
let loadRequestId = 0
const sortBy = ref('active_week')
const searchQuery = ref('')
const hasFlashcardsOnly = ref(false)
const currentPage = ref(1)
const perPage = ref(12)
const totalCollections = ref(0)
const showCreateModal = ref(false)
const isSubmitting = ref(false)
const importFileInput = ref(null)
const isImporting = ref(false)
const studyLoadingId = ref(null)
let searchDebounceTimer = null

const totalPages = computed(() => Math.max(1, Math.ceil(totalCollections.value / perPage.value)))

const sortOptions = computed(() => [
  {
    value: 'active_week',
    label: t('collectionList.sortActiveWeek'),
    icon: CalendarDays,
    aquaClass: 'ui-btn--sort-sky',
  },
  {
    value: 'active_month',
    label: t('collectionList.sortActiveMonth'),
    icon: Calendar,
    aquaClass: 'ui-btn--sort-blue',
  },
  {
    value: 'active_all',
    label: t('collectionList.sortActiveAll'),
    icon: Trophy,
    aquaClass: 'ui-btn--sort-amber',
  },
  {
    value: 'newest',
    label: t('collectionList.sortNewest'),
    icon: ArrowDown,
    aquaClass: 'ui-btn--sort-emerald',
  },
])

const selectedSortLabel = computed(
  () => sortOptions.value.find((o) => o.value === sortBy.value)?.label ?? ''
)

const newCollection = ref({
  name: '',
  description: '',
  is_public: true,
})

const pageTitle = ref(
  t(viewMode.value === 'my' ? 'collectionList.myCollections' : 'collectionList.publicCollections')
)
useSeoHead({ title: pageTitle, pathWithoutLocale: '/collections' })

const fetchCollections = async () => {
  const requestId = ++loadRequestId
  const isInitialLoad = !hasLoadedOnce.value
  if (isInitialLoad) {
    isLoading.value = true
  }

  try {
    // Only allow 'my' view mode when logged in
    if (!auth.state.isLoggedIn) {
      viewMode.value = 'public'
    }

    const params = {
      sort: sortBy.value,
      page: currentPage.value,
      per_page: perPage.value,
      has_flashcards_only: hasFlashcardsOnly.value ? true : undefined,
      search: searchQuery.value.trim() || undefined,
    }

    const response = await (viewMode.value === 'my' && auth.state.isLoggedIn
      ? getCollections(params)
      : getPublicCollections(params))

    if (requestId !== loadRequestId) return

    collections.value = response.data.collections || []
    totalCollections.value = Number(response.data.total || 0)

    if (collections.value.length === 0 && totalCollections.value > 0 && currentPage.value > 1) {
      currentPage.value = Math.max(1, totalPages.value)
      await fetchCollections()
      return
    }
  } catch (error) {
    if (requestId !== loadRequestId) return
    console.error('Error fetching collections:', error)
    totalCollections.value = 0
  } finally {
    if (requestId === loadRequestId) {
      isLoading.value = false
      hasLoadedOnce.value = true
    }
  }
}

const prevPage = () => {
  if (currentPage.value <= 1) return
  currentPage.value -= 1
  fetchCollections()
}

const nextPage = () => {
  if (currentPage.value >= totalPages.value) return
  currentPage.value += 1
  fetchCollections()
}

// Create new collection
const performCreateCollection = async () => {
  if (isSubmitting.value) return
  isSubmitting.value = true

  try {
    const response = await createCollection(newCollection.value)
    collections.value.unshift(response.data)
    showCreateModal.value = false
    newCollection.value = { name: '', description: '', is_public: true }
    router.push(`/collections/${response.data.collection_id}`)
  } catch (error) {
    console.error('Error creating collection:', error)
  } finally {
    isSubmitting.value = false
  }
}

const formatDate = (date) => {
  return new Date(date).toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const startStudy = async (collection) => {
  if (studyLoadingId.value) return
  studyLoadingId.value = collection.collection_id
  try {
    const response = await getLevels(collection.collection_id)
    const hasLevels = response.data?.levels?.length > 0
    if (hasLevels) {
      router.push(`/collections/${collection.collection_id}/levels`)
    } else {
      router.push(`/collections/${collection.collection_id}/flashcards/study`)
    }
  } catch (err) {
    console.error('Error loading levels:', err)
    router.push(`/collections/${collection.collection_id}/flashcards/study`)
  } finally {
    studyLoadingId.value = null
  }
}

const triggerImport = () => {
  importFileInput.value?.click()
}

const handleImportFile = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  isImporting.value = true
  try {
    const fileContent = await file.text()
    const jsonData = JSON.parse(fileContent)

    if (!jsonData.collection || !Array.isArray(jsonData.items)) {
      alert(t('collectionList.importFullFormatError'))
      return
    }

    const payload = {
      collection: {
        name: jsonData.collection.name,
        description: jsonData.collection.description ?? null,
        is_public: jsonData.collection.is_public ?? true,
      },
      items: jsonData.items,
      levels: Array.isArray(jsonData.levels) ? jsonData.levels : [],
    }

    const response = await importCollectionFull(payload)
    const { collection: newCollection } = response.data
    collections.value.unshift(newCollection)
    router.push(`/collections/${newCollection.collection_id}`)
  } catch (error) {
    console.error('Import failed:', error)
    alert(error.response?.data?.error || t('collectionList.importError'))
  } finally {
    isImporting.value = false
    event.target.value = ''
  }
}

function setViewMode(mode) {
  const next = mode === 'my' || mode === 'public' ? mode : viewMode.value
  if (!auth.state.isLoggedIn && next === 'my') return
  viewMode.value = next
  router.replace({ path: route.path, query: { ...route.query, [VIEW_PARAM]: next } })
}

// Persist viewMode to localStorage whenever it changes
watch(
  viewMode,
  (val) => {
    if (typeof localStorage !== 'undefined') localStorage.setItem(VIEW_STORAGE_KEY, val)
  },
  { immediate: true }
)

// Sync viewMode from URL when route query changes (e.g. back/forward)
watch(
  () => route.query[VIEW_PARAM],
  (qView) => {
    const next =
      validView(qView) && (qView === 'public' || auth.state.isLoggedIn)
        ? qView
        : auth.state.isLoggedIn
          ? 'my'
          : 'public'
    if (viewMode.value !== next) viewMode.value = next
  }
)

watch(
  () => auth.state.isLoggedIn,
  (loggedIn, wasLoggedIn) => {
    if (!loggedIn && wasLoggedIn) {
      collections.value = []
      hasLoadedOnce.value = false
    }
  }
)

// Watch for view mode or sort changes (and auth so collections load when auth resolves after mount)
watch([viewMode, sortBy, hasFlashcardsOnly, () => auth.state.isLoggedIn], () => {
  // Force public view when logged out
  if (!auth.state.isLoggedIn) {
    viewMode.value = 'public'
  }
  currentPage.value = 1
  fetchCollections()
})

watch(searchQuery, () => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    currentPage.value = 1
    fetchCollections()
  }, 300)
})

// Update title when view mode changes
watch(viewMode, (newMode) => {
  pageTitle.value = t(
    newMode === 'my' ? 'collectionList.myCollections' : 'collectionList.publicCollections'
  )
})

onMounted(() => {
  // Persist initial view to URL so refresh and back/forward stay in sync
  if (route.query[VIEW_PARAM] !== viewMode.value) {
    router.replace({ path: route.path, query: { ...route.query, [VIEW_PARAM]: viewMode.value } })
  }
  fetchCollections()
})

onBeforeUnmount(() => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
})
</script>

<style scoped>
.collections-section {
  @apply w-full;
}

.collections-grid {
  @apply grid gap-4 sm:gap-5 items-stretch grid-cols-1 sm:grid-cols-2 lg:grid-cols-3;
}

.animate-fade-in-up {
  animation: fadeInUp 0.3s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(1rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
