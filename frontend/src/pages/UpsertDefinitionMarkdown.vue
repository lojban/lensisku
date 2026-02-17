<template>
  <div class="container mx-auto p-4">
    <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none mb-4">
      {{ isEditMode ? t('upsertDefinitionMarkdown.editTitle') : t('upsertDefinitionMarkdown.addTitle') }}
    </h2>

    <form
      class="space-y-4"
      @submit.prevent="submitValsi"
    >
      <!-- Word Input -->
      <div>
        <label
          for="word"
          class="block text-sm font-medium text-blue-700"
        > {{ t('upsertDefinitionMarkdown.wordLabel') }} </label>
        <input
          id="word"
          v-model="word"
          type="text"
          required
          class="input-field w-full h-10 mb-4"
          :disabled="isSubmitting"
          :placeholder="t('upsertDefinitionMarkdown.wordPlaceholder')"
        >
      </div>

      <!-- Language Selection -->
      <div>
        <label
          for="language"
          class="block text-sm font-medium text-blue-700"
        > {{ t('upsertDefinitionMarkdown.languageLabel') }} </label>
        <select
          id="language"
          v-model="langId"
          required
          class="input-field w-full h-10"
          :disabled="isLoading || isSubmitting"
        >
          <option value="">
            {{ t('upsertDefinitionMarkdown.languagePlaceholder') }}
          </option>
          <option
            v-for="lang in languages"
            :key="lang.id"
            :value="lang.id"
          >
            {{ lang.real_name }} ({{ lang.english_name }})
          </option>
        </select>
      </div>

      <!-- Entry Language Selection (Only for new entries) -->
      <div v-if="!isEditMode">
        <label for="source-language" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.sourceLanguageLabel', 'Entry Language') }} <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
        <select id="source-language" v-model="sourceLangId" required class="input-field w-full h-10" :disabled="isLoading || isSubmitting || isEditMode">
          <option value="">{{ t('upsertDefinition.selectLanguagePlaceholder') }}</option>
          <option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.real_name }} ({{ lang.english_name }})
          </option>
        </select>
         <p class="mt-1 text-xs text-gray-500">
          {{ t('upsertDefinition.sourceLanguageNote', 'The language the word itself belongs to. Cannot be changed after creation.') }}
        </p>
      </div>

      <!-- Definition Editor -->
      <div>
        <label class="block text-sm font-medium text-blue-700 mb-2"> {{ t('upsertDefinitionMarkdown.definitionLabel') }} </label>
        <div
          ref="editor"
          class="milkdown-editor"
        />
      </div>

      <!-- Submit Button -->
      <div class="flex justify-end">
        <button
          type="submit"
          class="btn-create"
          :disabled="isSubmitting || !isValid"
        >
          {{ isSubmitting ? t('upsertDefinitionMarkdown.saving') : t('upsertDefinitionMarkdown.saveButton') }}
        </button>
      </div>
    </form>
  </div>
</template>

<script setup>
import { Crepe } from '@milkdown/crepe';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

import { getLanguages, addValsi, updateValsi, getDefinition } from '@/api';
import '@milkdown/crepe/theme/common/style.css';
import '@milkdown/crepe/theme/frame.css';
import { useAuth } from '@/composables/useAuth';

const route = useRoute();
const router = useRouter();
const auth = useAuth();
const { t } = useI18n();

// Form state
const langId = ref('')
const word = ref('')
const definition = ref('')
const sourceLangId = ref(1)
const editor = ref(null)
let crepe = null

onMounted(async () => {
  await loadLanguages()

  const definitionId = route.params.id
  if (definitionId) {
    isEditMode.value = true
    editDefinitionId.value = definitionId
    await loadDefinitionData(definitionId)
  }

  // Initialize Crepe editor
  crepe = new Crepe({
    root: editor.value,
    defaultValue: definition.value,
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: 'Type / to show menu',
      },
      [Crepe.Feature.ImageBlock]: {
        onUpload: async (file) => {
          if (typeof window === 'undefined') return;
          // Convert file to base64
          const reader = new FileReader()
          reader.readAsDataURL(file)
          const dataUrl = await new Promise(
            (resolve) => (reader.onload = () => resolve(reader.result))
          )

          // Extract mime type and data
          const [mimeType, data] = dataUrl.split(',', 2)

          // Use definition ID from current context
          const response = await fetch(
            `/api/jbovlaste/definition_image/${editDefinitionId.value}/image`,
            {
              method: 'POST',
              headers: {
                Authorization: `Bearer ${localStorage.getItem('accessToken')}`,
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                image: {
                  data: data,
                  mime_type: mimeType.replace('data:', '').split(';')[0],
                },
              }),
            }
          )

          const result = await response.json()
          console.log(result)
          if (!response.ok) throw new Error(result.error)

          return `/api/jbovlaste/definition_image/${editDefinitionId.value}/image?image_id=${result.image_id}`
        },
      },
    },
  })

  await crepe.create()

  // Update definition ref on change
  crepe.on((listener) => {
    listener.markdownUpdated(() => {
      definition.value = crepe.getMarkdown()
    })
  })
})

onUnmounted(() => {
  if (crepe) {
    crepe.destroy()
  }
})
const languages = ref([])
const isEditMode = ref(false)
const isSubmitting = ref(false)
const isLoading = ref(true)
const editDefinitionId = ref(null)

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

async function loadDefinitionData(definitionId) {
  try {
    const response = await getDefinition(definitionId)
    const def = response.data
    if (def) {
      langId.value = def.langid
      definition.value = def.definition
      word.value = def.valsiword
      sourceLangId.value = def.source_langid || 1 // Load source lang if editing
    }
  } catch (error) {
    console.error('Failed to load definition:', error)
  }
}

async function submitValsi() {
  if (!isValid.value) return

  isSubmitting.value = true

  try {
    const requestData = {
      word: word.value,
      definition: definition.value, // This already sends the markdown content
      notes: null,
      etymology: null,
      lang_id: parseInt(langId.value),
      // Only include source_langid when adding a new definition
      ...( !isEditMode.value && { source_langid: parseInt(sourceLangId.value) || 1 } ),
      selmaho: null,
      jargon: null,
      gloss_keywords: null,
      place_keywords: null,
      owner_only: false,
      image: null,
    }

    let response
    if (isEditMode.value) {
      response = await updateValsi(editDefinitionId.value, requestData)
    } else {
      response = await addValsi(requestData)
    }

    if (response.data.success || response.status === 200) { // Check for 200 status as well
      const definitionId = response.data.definition_id || editDefinitionId.value
      // Redirect to the entry page after successful save
      router.push(`/valsi/${word.value.replace(/ /g, '_')}?highlight_definition_id=${definitionId}`)
    } else {
       console.error('Error saving definition:', response.data.error)
       // Potentially show error to user using useError composable if needed
    }
  } catch (error) {
     console.error('Error saving definition:', error)
     // Potentially show error to user using useError composable if needed
  } finally {
    isSubmitting.value = false
  }
}

onMounted(async () => {
  await loadLanguages()

  const definitionId = route.params.id
  if (definitionId) {
    isEditMode.value = true
    editDefinitionId.value = definitionId
    await loadDefinitionData(definitionId)
  }
})
</script>

<style scoped>
.container {
  max-width: 800px;
}

.milkdown-editor {
  @apply border border-gray-300;
  /* min-height: 300px; */
}
</style>
