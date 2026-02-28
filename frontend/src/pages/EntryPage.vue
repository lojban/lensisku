<template>
  <!-- Loading State -->
  <div
    v-if="isLoading"
    class="flex justify-center py-8"
  >
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
  </div>

  <!-- Content: semantic article for SEO and accessibility -->
  <article
    v-else-if="valsi"
    class="space-y-3"
    itemscope
    itemtype="https://schema.org/DefinedTerm"
  >
    <!-- Header Section -->
    <h1 class="border-b p-2 min-w-0 rounded-md">
      <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-between min-w-0">
        <div class="flex items-center gap-3 min-w-0 max-w-full">
          <h1 class="text-3xl font-bold text-gray-800 min-w-0 max-w-full break-words" itemprop="name">
            {{ valsi.word }}
            <AudioPlayer
              v-if="definitions.length > 0 && definitions[0].sound_url"
              :url="definitions[0].sound_url"
              :valsi-id-or-word="route.params.id"
              class="flex-shrink-0"
            />
          </h1>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <span
            class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium"
            :class="getTypeClass(valsi.type_name)"
          >
            {{ getWordTypeLabel(valsi.type_name) }}
          </span>
          <span
            v-if="valsi.rafsi"
            class="inline-flex items-center px-3 py-1 bg-gray-100 rounded-full text-sm font-medium text-gray-700"
          >
            {{ t('entryPage.rafsiLabel') }} {{ valsi.rafsi }}
          </span>
          <div
            v-if="valsi.decomposition"
            class="inline-flex items-center gap-1 text-sm text-gray-700"
          >
            <span class="font-medium">{{ t('entryPage.decompositionLabel') }}</span>
            <template
              v-for="(word, index) in valsi.decomposition"
              :key="word"
            >
              <RouterLink 
                :to="{ path: `/valsi/${word.replace(/ /g, '_')}`, query: { langid: valsi.source_langid } }"
                class="text-blue-600 hover:text-blue-800 hover:underline"
              >
                {{ word }}
              </RouterLink>
              <span v-if="index < valsi.decomposition.length - 1">+</span>
            </template>
          </div>
        </div>
      </div>
    </h1>

    <!-- Discussion Section -->
    <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-center">
      <RouterLink
        :to="`/comments?valsi_id=${valsi.valsiid}`"
        class="btn-aqua-slate"
      >
        <AudioWaveform class="w-4 h-4" />
        <span
          v-if="(valsi.comment_count ?? 0) > 0"
          class="text-xs font-medium bg-white/40 px-1.5 rounded-md border border-white/30"
        >
          {{ valsi.comment_count }}
        </span>
        <span>{{ t('entryPage.discussEntry') }}</span>
      </RouterLink>
      <SubscriptionControls
        :valsi-id="valsi.valsiid"
        :auth="auth"
      />
    </div>

    <!-- Definitions Section -->
    <div class="space-y-4">
      <h3 class="text-xl font-semibold text-gray-700 flex items-center gap-2">
        <span>{{ t('entryPage.definitions') }}</span>
        <span class="text-sm font-normal text-gray-500">({{ definitions.length }})</span>
      </h3>

      <div class="space-y-4">
        <DefinitionCard
          v-for="def in definitions"
          :key="def.definitionid"
          :definition="def"
          :languages="languages"
          :disable-discussion-button="true"
          :show-score="props.showScores"
          :show-comment-button="false"
          :show-word-type="false"
          :show-audio="false"
          :definition-id="def.definitionid"
          :show-definition-number="true" 
          :collections="collections"
          @refresh-definitions="fetchDefinitionsDetails"
        />
      </div>

      <!-- No Definitions State -->
      <div
        v-if="definitions.length === 0"
        class="text-center py-8 bg-gray-50 rounded-lg text-gray-600"
      >
        {{ t('entryPage.noDefinitions') }}
      </div>
    </div>


    <!-- Translations Section -->
    <div v-if="translations.length > 0" class="space-y-4 pt-4 border-t">
      <h3 class="text-xl font-semibold text-gray-700 flex items-center gap-2">
        <span>{{ t('entryPage.translationsLabel') }}</span>
        <span class="text-sm font-normal text-gray-500">({{ translations.length }})</span>
      </h3>

      <div class="space-y-4">
        <div v-for="trans in translations" :key="trans.definitionid" class="p-3 bg-gray-50 rounded-lg border">
          <div class="flex justify-between items-start">
            <div>
              <RouterLink 
                :to="`/valsi/${trans.valsiword}?highlight_definition_id=${trans.definitionid}`"
                class="font-bold text-blue-600 hover:underline"
              >
                {{ trans.valsiword }}
              </RouterLink>
              <span class="text-gray-500 text-sm ml-2">({{ trans.lang_name }})</span>
              <div class="mt-1 text-gray-800">{{ trans.definition }}</div>
              <div class="mt-2 flex items-center space-x-3">
                <RouterLink 
                  :to="`/definition_link/${trans.link_id}/discussion`"
                  class="text-xs font-bold text-blue-500 hover:text-blue-700 flex items-center bg-blue-50 px-2 py-1 rounded transition-colors"
                >
                  <MessageCircle class="h-3 w-3 mr-1" />
                  {{ t('definitionLinkDiscussion.discussion') }}
                </RouterLink>
              </div>
            </div>
            <button 
              v-if="auth.state.isLoggedIn"
              @click="unlinkTranslation(trans)"
              class="text-red-500 hover:text-red-700 p-1"
              :title="t('entryPage.unlinkTranslation')"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-wrap gap-3 pt-4 border-t">
      <button
        class="btn-aqua-zinc"
        @click="goBack"
      >
        <ArrowLeft class="h-5 w-5" />
        <span>{{ t('entryPage.dictionary') }}</span>
      </button>

      <IconButton
        v-if="auth.state.isLoggedIn"
        :label="t('entryPage.addDefinition')"
        button-classes="btn-aqua-emerald"
        @click="router.push(`/valsi/add?word=${encodeURIComponent(valsi.word)}`)"
      />
    </div>
  </article>
</template>

<script setup>
import { ArrowLeft, AudioWaveform, Trash2, MessageCircle } from 'lucide-vue-next'
import { ref, onMounted, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { getTypeClass } from '@/utils/wordTypeUtils'; // Import shared utility

import { getValsiDefinitions, getValsiDetails, getCollections, getLanguages, getDefinitionTranslations, unlinkDefinitions } from '@/api'
import AudioPlayer from '@/components/AudioPlayer.vue'
import DefinitionCard from '@/components/DefinitionCard.vue'
import IconButton from '@/components/icons/IconButton.vue'
import SubscriptionControls from '@/components/SubscriptionControls.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const route = useRoute()

const props = defineProps({
  isPreferred: {
    type: Boolean,
    default: false,
  },
  onRefresh: {
    type: Function,
    default: () => { },
  },
})

const router = useRouter()
const auth = useAuth()
const { t, locale } = useI18n()

const languages = ref([])

const collections = ref([])
const fetchCollections = async () => {
  try {
    const response = await getCollections()
    collections.value = response.data.collections
  } catch (error) {
    console.error('Error fetching collections:', error)
  }
}

const valsi = ref(null)
const definitions = ref([])
const isLoading = ref(true)
const { showError, clearError } = useError()

const entryTitle = computed(() => valsi.value?.word || t('entryPage.entry'))
const entryDescription = computed(() => {
  const v = valsi.value
  if (!v) return ''
  const typeLabel = getWordTypeLabel(v.type_name)
  return t('entryPage.metaDescription', { word: v.word, type: typeLabel })
})
const entryCanonical = computed(() => route.fullPath)

useSeoHead({
  title: entryTitle,
  description: entryDescription,
  canonical: entryCanonical,
})

// Remove the local getTypeClass implementation

const fetchDefinitionsDetails = async () => {
  isLoading.value = true
  clearError()

  try {
    const valsiId = route.params.id
    const [valsiRes, defsRes] = await Promise.all([
      getValsiDetails(valsiId),
      getValsiDefinitions(valsiId),
    ])

    valsi.value = valsiRes.data.valsi
    definitions.value = defsRes.data
  } catch (e) {
    if (e.response?.status === 404) {
      router.push({
        path: '/valsi/add',
        query: {
          word: route.params.id,
          langid: route.query.langid,
          username: route.query.username,
        },
      })
      return
    }
    showError(e.response?.data || t('entryPage.loadError')) // Use t()
    console.error('Error fetching valsi details:', e)
  } finally {
    isLoading.value = false
  }
}

const goBack = () => {
  router.back()
}

// Watch for route changes to handle navigation between different valsi
watch(
  () => route.params.id,
  async (newId, oldId) => {
    if (newId !== oldId) {
      await fetchDefinitionsDetails()
    }
  }
)

watch(
  () => route.query.highlight_definition_id,
  (newVal) => {
    if (newVal) {
      scrollToDefinition()
    }
  }
)

const highlightedDefinitionId = computed(() => route.query.highlight_definition_id)

const scrollToDefinition = () => {
  const definitionId = route.query.highlight_definition_id
  if (definitionId) {
    setTimeout(() => {
      const element = document.querySelector(`[data-definition-id="${definitionId}"]`)
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'center' })
        element.classList.add('highlight-definition')
        setTimeout(() => element.classList.remove('highlight-definition'), 5800)
      }
    }, 100)
  }
}

const getWordTypeLabel = (typeName) => {
  if (!typeName) return ''
  const key = `wordTypes.${typeName.replace(/'/g,'h').replace(/ /g, '-')}`
  const translated = t(key)
  // If translation returns the key itself, it means the translation doesn't exist
  // In that case, return the type name as fallback (except for experimental cmavo which is ok)
  if (translated === key && typeName !== 'experimental cmavo') {
    return typeName
  }
  return translated
}

onMounted(async () => {
  await Promise.all([fetchDefinitionsDetails(), fetchCollections()])
  if (highlightedDefinitionId.value) {
    scrollToDefinition()
  }
  const langsResponse = await getLanguages()
  languages.value = langsResponse.data
})

const translations = ref([])

const fetchTranslations = async () => {
  // Only fetch translations for phrases
  if (valsi.value?.type_name !== 'phrase') {
    translations.value = []
    return
  }

  if (definitions.value.length > 0) {
    const allTranslations = []
    for (const def of definitions.value) {
      try {
        const res = await getDefinitionTranslations(def.definitionid)
        if (res.data && res.data.length > 0) {
          allTranslations.push(...res.data.map(t => ({ ...t, source_def_id: def.definitionid })))
        }
      } catch (e) {
        console.error('Error fetching translations for def', def.definitionid, e)
      }
    }
    // Deduplicate by definitionid
    translations.value = Array.from(new Map(allTranslations.map(item => [item.definitionid, item])).values())
  }
}

const unlinkTranslation = async (translation) => {
  if (!confirm(t('entryPage.confirmUnlink'))) return
  
  try {
    await unlinkDefinitions(translation.source_def_id, translation.definitionid)
    await fetchTranslations()
  } catch (e) {
    console.error('Error unlinking definition:', e)
    showError(t('entryPage.unlinkError'))
  }
}

watch(definitions, async (newDefs) => {
  if (newDefs.length > 0) {
    await fetchTranslations()
  } else {
    translations.value = []
  }
})
</script>

<style scoped>
/* Make sure MathJax content is properly contained */
:deep(.mathjax-content) {
  overflow-x: auto;
  min-height: 1em;
}

/* Improve MathJax display */
:deep(.MathJax) {
  margin: 0.5em 0;
}

:deep(.MathJax_Display) {
  margin: 1em 0;
}

@keyframes highlight-definition {

  0%,
  95% {
    @apply outline outline-orange-600 outline-2 bg-orange-50 border-orange-600;
  }

  100% {
    background-color: transparent;
    box-shadow: none;
  }
}

.highlight-definition {
  animation: highlight-definition 5.8s ease-out;
}
</style>
