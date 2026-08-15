<template>
  <div class="wiki-upsert w-full max-w-3xl mx-auto px-3 py-3 sm:px-4 sm:py-4">
    <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none mb-4">
      {{ isEditMode ? t('upsertWiki.editTitle') : t('upsertWiki.addTitle') }}
    </h2>

    <form class="space-y-4" @submit.prevent="submitWiki">
      <!-- Word Input -->
      <div>
        <label for="word" class="block text-sm font-medium text-blue-700">
          {{ t('upsertWiki.wordLabel') }}
        </label>
        <Input
          id="word"
          v-model="word"
          type="text"
          required
          class="input-field w-full h-10"
          :disabled="isSubmitting || isEditMode"
          :placeholder="t('upsertWiki.wordPlaceholder')"
        />
      </div>

      <!-- Languages: stacked on mobile, one row on md+ when both shown -->
      <div
        class="grid grid-cols-1 gap-4"
        :class="{ 'md:grid-cols-2': !isEditMode }"
      >
        <div v-if="!isEditMode">
          <label for="source-language" class="block text-sm font-medium text-blue-700">
            {{ t('upsertWiki.sourceLanguageLabel') }}
            <span class="text-red-500">{{ t('upsertWiki.required') }}</span>
          </label>
          <Select
            id="source-language"
            v-model="sourceLangId"
            required
            class="input-field w-full h-10"
            :disabled="isLoading || isSubmitting"
            :options="[
              { value: '', label: t('upsertWiki.selectLanguagePlaceholder') },
              ...languages.map((lang) => ({ value: lang.id, label: lang.real_name })),
            ]"
          />
          <p class="mt-1 text-xs text-gray-500">{{ t('upsertWiki.sourceLanguageNote') }}</p>
        </div>

        <div>
          <label for="language" class="block text-sm font-medium text-blue-700">
            {{ t('upsertWiki.languageLabel') }}
          </label>
          <Select
            id="language"
            v-model="langId"
            required
            class="input-field w-full h-10"
            :disabled="isLoading || isSubmitting"
            :options="[
              { value: '', label: t('upsertWiki.languagePlaceholder') },
              ...languages.map((lang) => ({ value: lang.id, label: lang.real_name })),
            ]"
          />
        </div>
      </div>

      <!-- Definition Editor: full-bleed within page padding on mobile -->
      <div>
        <label class="block text-sm font-medium text-blue-700 mb-2">
          {{ t('upsertWiki.definitionLabel') }}
        </label>
        <div
          ref="editor"
          class="milkdown-editor -mx-3 border-y border-gray-300 sm:-mx-4 sm:border"
        />
      </div>

      <!-- Submit Button -->
      <div class="flex justify-end">
        <Button variant="create" type="submit" :disabled="isSubmitting || !isValid">
          <template v-if="isSubmitting">{{ t('upsertWiki.saving') }}<AnimatedDots /></template>
          <template v-else>{{ t('upsertWiki.saveButton') }}</template>
        </Button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { Button, Input, Select } from '@packages/ui'
import { Crepe } from '@milkdown/crepe'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { getLanguages, addValsi, updateValsi, getNativeWikiArticle } from '@/api'
import AnimatedDots from '@/components/AnimatedDots.vue'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'
import { useSeoHead } from '@/composables/useSeoHead'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const langId = ref('')
const word = ref('')
const sourceLangId = ref(1)
const definition = ref('')
const editor = ref<HTMLElement | null>(null)
let crepe: Crepe | null = null

onMounted(async () => {
  await loadLanguages()

  const wordParam = route.params.word as string | undefined
  if (wordParam) {
    isEditMode.value = true
    word.value = decodeURIComponent(wordParam).replace(/_/g, ' ')
    await loadWikiData(word.value)
  }

  crepe = new Crepe({
    root: editor.value,
    defaultValue: definition.value,
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: t('upsertWiki.editorPlaceholder') || 'Type / to show menu',
      },
    },
  })

  await crepe.create()

  const updateDefinition = () => {
    if (crepe) {
      let markdown = crepe.getMarkdown()
      // Convert autolinks <https://...> to [https://...](https://...)
      markdown = markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
      definition.value = markdown
    }
  }

  crepe.on((listener) => {
    listener.markdownUpdated(updateDefinition)
  })
})

onUnmounted(() => {
  if (crepe) {
    crepe.destroy()
  }
})

const languages = ref<{ id: number; real_name: string }[]>([])
const isEditMode = ref(false)
const isSubmitting = ref(false)
const isLoading = ref(true)
const editDefinitionId = ref<number | null>(null)

const pageTitle = computed(() =>
  isEditMode.value ? t('upsertWiki.editTitle') : t('upsertWiki.addTitle')
)
useSeoHead({ title: pageTitle, robots: 'noindex, nofollow' })

const isValid = computed(() => {
  return langId.value && definition.value.trim()
})

async function loadLanguages() {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (error) {
    console.error('Failed to load languages:', error)
  } finally {
    isLoading.value = false
  }
}

async function loadWikiData(wikiWord: string) {
  try {
    const response = await getNativeWikiArticle(wikiWord)
    const def = response.data
    if (def) {
      langId.value = String(def.langid)
      sourceLangId.value = def.source_langid || 1
      definition.value = def.definition
      editDefinitionId.value = def.definitionid
    }
  } catch (error) {
    console.error('Failed to load wiki data:', error)
  }
}

async function submitWiki() {
  if (crepe) {
    let markdown = crepe.getMarkdown()
    markdown = markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
    definition.value = markdown
  }
  if (!isValid.value) return

  isSubmitting.value = true

  try {
    const requestData = {
      word: word.value,
      definition: definition.value,
      notes: null,
      etymology: null,
      lang_id: parseInt(String(langId.value), 10),
      ...(!isEditMode.value && {
        source_langid: parseInt(String(sourceLangId.value), 10) || 1,
      }),
      selmaho: null,
      jargon: null,
      gloss_keywords: null,
      place_keywords: null,
      owner_only: false,
      image: null,
      is_wiki: true,
    }

    let response
    if (isEditMode.value && editDefinitionId.value !== null) {
      response = await updateValsi(editDefinitionId.value, requestData)
    } else {
      response = await addValsi(requestData)
    }

    if (response.data.success || response.status === 200) {
      router.push(`/wiki/${word.value.replace(/ /g, '_')}`)
    } else {
      console.error('Error saving wiki page:', response.data.error)
    }
  } catch (error) {
    console.error('Error saving wiki page:', error)
  } finally {
    isSubmitting.value = false
  }
}
</script>

<style scoped>
.milkdown-editor {
  @apply min-h-60 flex flex-col;
}

/* Crepe nests .milkdown > .ProseMirror; both must stretch to the host min-height. */
.milkdown-editor :deep(.milkdown) {
  @apply flex min-h-full flex-1 flex-col;
}

.milkdown-editor :deep(.milkdown .ProseMirror) {
  @apply min-h-full flex-1 py-3 px-3 sm:py-4 sm:pl-14 sm:pr-4;
}

/* Block handle (drag/+/menu) is hover-based and wastes width on small screens. */
@media (max-width: 640px) {
  .milkdown-editor :deep(.milkdown-block-handle),
  .milkdown-editor :deep(milkdown-block-handle) {
    display: none !important;
  }
}
</style>
