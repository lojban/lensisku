<template>
  <UpsertPageLayout
    narrow
    :title="isEditMode ? t('upsertWiki.editTitle') : t('upsertWiki.addTitle')"
  >
    <template #trailing>
      <UpsertToolbarButton
        type="submit"
        form="upsert-wiki-form"
        :variant="isEditMode ? 'edit' : 'create'"
        :disabled="isSubmitting || !isValid"
      >
        <template v-if="isSubmitting">{{ t('upsertWiki.saving') }}<AnimatedDots /></template>
        <template v-else>{{ t('upsertWiki.saveButton') }}</template>
      </UpsertToolbarButton>
    </template>

    <div
      v-if="fromCommentId"
      class="mb-4 p-3 bg-blue-50 border border-blue-100 rounded text-sm text-blue-800"
    >
        {{ t('upsertWiki.fromCommentBanner', { id: fromCommentId }) }}
        <RouterLink
          :to="`/comments?comment_id=${fromCommentId}&scroll_to=${fromCommentId}`"
          class="ml-1 underline hover:text-blue-950"
        >
          {{ t('upsertWiki.fromCommentLink') }}
        </RouterLink>
      </div>

      <form id="upsert-wiki-form" class="space-y-4" @submit.prevent="submitWiki">
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
            :disabled="isSubmitting || isGeneratingTitle"
            :placeholder="t('upsertWiki.wordPlaceholder')"
          />
          <p v-if="isGeneratingTitle" class="mt-1 text-xs text-gray-500">
            {{ t('upsertWiki.generatingTitle') }}
          </p>
          <p v-if="isEditMode && titleChanged" class="mt-1 text-xs text-amber-700">
            {{ t('upsertWiki.renameHint') }}
          </p>
        </div>

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

        <div>
          <label for="commit-message" class="block text-sm font-medium text-blue-700">
            {{ t('upsertWiki.commitMessageLabel') }}
          </label>
          <Input
            id="commit-message"
            v-model="commitMessage"
            type="text"
            class="input-field w-full h-10"
            :disabled="isSubmitting"
            :placeholder="t('upsertWiki.commitMessagePlaceholder')"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-blue-700 mb-2">
            {{ t('upsertWiki.definitionLabel') }}
          </label>
          <WikiEditor
            v-if="editorReady"
            :key="editorKey"
            ref="wikiEditor"
            v-model="definition"
            :disabled="isSubmitting"
            :placeholder="t('upsertWiki.editorPlaceholder')"
          />
        </div>

        <p v-if="formError" class="text-sm text-red-600">{{ formError }}</p>
      </form>
  </UpsertPageLayout>
</template>

<script setup lang="ts">
import { Input, Select } from '@packages/ui'
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { getLanguages, addValsi, updateValsi, getNativeWikiArticle, renameWiki, suggestWikiTitleFromComment } from '@/api'
import AnimatedDots from '@/components/AnimatedDots.vue'
import UpsertPageLayout from '@/components/layout/UpsertPageLayout.vue'
import UpsertToolbarButton from '@/components/layout/UpsertToolbarButton.vue'
import WikiEditor from '@/components/WikiEditor.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { loadWikiFromCommentPrefill } from '@/utils/wikiFromComment'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const langId = ref('')
const word = ref('')
const originalWord = ref('')
const sourceLangId = ref(1)
const definition = ref('')
const commitMessage = ref('')
const expectedTime = ref<number | null>(null)
const wikiEditor = ref<{ getMarkdown: () => string } | null>(null)
const editorReady = ref(false)
const editorKey = ref(0)
const formError = ref('')
const fromCommentId = ref<number | null>(null)
const isGeneratingTitle = ref(false)

const languages = ref<{ id: number; real_name: string }[]>([])
const isEditMode = ref(false)
const isSubmitting = ref(false)
const isLoading = ref(true)
const editDefinitionId = ref<number | null>(null)

const pageTitle = computed(() =>
  isEditMode.value ? t('upsertWiki.editTitle') : t('upsertWiki.addTitle')
)
useSeoHead({ title: pageTitle, robots: 'noindex, nofollow' })

const titleChanged = computed(
  () => isEditMode.value && word.value.trim() !== originalWord.value.trim()
)

const isValid = computed(() => {
  return langId.value && word.value.trim() && definition.value.trim()
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
      originalWord.value = def.valsiword
      word.value = def.valsiword
      expectedTime.value = typeof def.time === 'number' ? def.time : null
      editorKey.value += 1
    }
  } catch (error) {
    console.error('Failed to load wiki data:', error)
    formError.value = t('upsertWiki.loadError')
  }
}

async function submitWiki() {
  formError.value = ''
  if (wikiEditor.value) {
    definition.value = wikiEditor.value.getMarkdown()
  }
  if (!isValid.value) return

  if (titleChanged.value) {
    const confirmed = window.confirm(
      t('upsertWiki.renameConfirm', { oldTitle: originalWord.value, newTitle: word.value.trim() })
    )
    if (!confirmed) return
  }

  isSubmitting.value = true

  try {
    if (isEditMode.value && editDefinitionId.value !== null && titleChanged.value) {
      const renameResp = await renameWiki(editDefinitionId.value, {
        new_word: word.value.trim(),
      })
      if (!renameResp.data.success) {
        formError.value = renameResp.data.error || t('upsertWiki.renameError')
        return
      }
      originalWord.value = renameResp.data.new_word
      word.value = renameResp.data.new_word
      // Rename bumps definitions.time; skip stale concurrency check for the follow-up save.
      expectedTime.value = null
    }

    const requestData = {
      word: word.value.trim(),
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
      commit_message: commitMessage.value.trim() || undefined,
      expected_time: expectedTime.value ?? undefined,
      ...(!isEditMode.value &&
        fromCommentId.value && {
          metadata: {
            source_comment_id: fromCommentId.value,
          },
        }),
    }

    let response
    if (isEditMode.value && editDefinitionId.value !== null) {
      response = await updateValsi(editDefinitionId.value, requestData)
    } else {
      response = await addValsi(requestData)
    }

    if (response.data.success || response.status === 200) {
      router.push(`/wiki/${word.value.trim().replace(/ /g, '_')}`)
    } else {
      formError.value = response.data.error || t('upsertWiki.saveError')
    }
  } catch (error: unknown) {
    const status = (error as { response?: { status?: number; data?: { error?: string } } })
      ?.response?.status
    const apiError = (error as { response?: { data?: { error?: string } } })?.response?.data
      ?.error
    if (status === 409) {
      formError.value = apiError || t('upsertWiki.conflictError')
    } else {
      formError.value = apiError || t('upsertWiki.saveError')
      console.error('Error saving wiki page:', error)
    }
  } finally {
    isSubmitting.value = false
  }
}

async function applyWikiTitleFromComment(commentId: number, fallbackTitle = '') {
  if (fallbackTitle) {
    word.value = fallbackTitle
  }
  isGeneratingTitle.value = true
  try {
    const response = await suggestWikiTitleFromComment(commentId)
    if (response.data?.title?.trim()) {
      word.value = response.data.title.trim()
    }
  } catch (error) {
    console.warn('Failed to suggest wiki title from comment:', error)
  } finally {
    isGeneratingTitle.value = false
  }
}

async function loadFromCommentPrefill(fromCommentQuery: string) {
  const prefill = loadWikiFromCommentPrefill(String(fromCommentQuery))
  if (prefill) {
    fromCommentId.value = prefill.commentId
    definition.value = prefill.definition || ''
    commitMessage.value =
      prefill.commitMessage || t('upsertWiki.fromCommentCommit', { id: prefill.commentId })
    editorKey.value += 1
    await applyWikiTitleFromComment(prefill.commentId, prefill.word)
    return
  }

  const commentId = Number(fromCommentQuery) || null
  fromCommentId.value = commentId
  if (commentId) {
    await applyWikiTitleFromComment(commentId)
  }
}

onMounted(async () => {
  await loadLanguages()

  const wordParam = route.params.word as string | undefined
  if (wordParam) {
    isEditMode.value = true
    word.value = decodeURIComponent(wordParam).replace(/_/g, ' ')
    originalWord.value = word.value
    await loadWikiData(word.value)
  } else {
    const fromCommentQuery = route.query.from_comment
    if (fromCommentQuery) {
      await loadFromCommentPrefill(String(fromCommentQuery))
    }
  }

  editorReady.value = true
})
</script>
