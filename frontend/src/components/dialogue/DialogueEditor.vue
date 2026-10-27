<template>
  <div class="dialogue-editor">
    <div class="flex items-center justify-between gap-2">
      <label :for="textareaId" class="block text-sm font-medium text-gray-700">
        {{ t('dialogue.editor.scriptLabel') }}
      </label>
      <button type="button" class="text-xs text-blue-700 underline" @click="showHelp = !showHelp">
        {{ showHelp ? t('dialogue.editor.hideSyntax') : t('dialogue.editor.showSyntax') }}
      </button>
    </div>
    <pre v-if="showHelp" class="dialogue-editor__help">{{ t('dialogue.editor.syntax') }}</pre>
    <textarea
      :id="textareaId"
      :value="modelValue"
      rows="12"
      class="textarea-field dialogue-editor__textarea"
      spellcheck="false"
      :placeholder="t('dialogue.editor.placeholder')"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />

    <div v-if="parsing" class="text-xs text-gray-500">{{ t('dialogue.editor.parsing') }}</div>
    <ul v-if="errors.length" class="dialogue-editor__errors" role="alert">
      <li v-for="(e, i) in errors" :key="i">
        {{ t('dialogue.editor.errorAt', { line: e.line, message: e.message }) }}
      </li>
    </ul>
    <p v-else-if="parsed" class="text-xs text-gray-600">
      {{
        t('dialogue.editor.stats', {
          lines: parsed.dialogue.lines.length,
          questions: parsed.dialogue.questions.length,
        })
      }}
    </p>

    <!-- Speakers → voices -->
    <div v-if="speakers.length" class="dialogue-editor__voices">
      <div v-for="sp in speakers" :key="sp" class="dialogue-editor__voice">
        <span class="font-medium">{{ sp === NARRATOR ? t('dialogue.narrator') : sp }}</span>
        <select v-model="voiceMap[sp]" class="input-field text-sm" :aria-label="sp">
          <option v-for="v in DIALOGUE_VOICES" :key="v" :value="v">{{ v }}</option>
        </select>
      </div>
    </div>

    <!-- Per-line audio -->
    <div v-if="parsed?.dialogue.lines.length" class="dialogue-editor__lines">
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-sm font-medium text-gray-700">{{ t('dialogue.editor.audio') }}</span>
        <button
          type="button"
          class="lingo-btn lingo-btn--primary lingo-btn--sm !normal-case !tracking-normal"
          :disabled="!canGenerate || generating"
          @click="generate(null, false)"
        >
          <LoadingSpinner v-if="generating && generatingLine == null" size="sm" />
          {{ t('dialogue.editor.generateAll') }}
        </button>
        <span v-if="!itemId" class="text-xs text-gray-500">{{
          t('dialogue.editor.saveFirst')
        }}</span>
        <span v-else-if="dirty" class="text-xs text-amber-700">{{
          t('dialogue.editor.unsaved')
        }}</span>
      </div>
      <p v-if="audioError" class="text-xs text-red-600" role="alert">{{ audioError }}</p>
      <ol class="dialogue-editor__line-list">
        <li v-for="line in parsed.dialogue.lines" :key="line.index" class="dialogue-editor__line">
          <span class="dialogue-editor__line-speaker">{{ line.speaker }}</span>
          <span class="dialogue-editor__line-text" lang="jbo">{{ line.text }}</span>
          <span class="dialogue-editor__line-voice">{{ voiceForLine(line) }}</span>
          <button
            type="button"
            class="dialogue-editor__icon"
            :disabled="!lineHasAudio(line.index)"
            :aria-label="t('dialogue.playLine')"
            @click="audio.play(line.index, audioVersion)"
          >
            <Volume2 class="h-4 w-4" aria-hidden="true" />
          </button>
          <button
            type="button"
            class="dialogue-editor__icon"
            :disabled="!canGenerate || generating"
            :aria-label="t('dialogue.editor.regenerateLine')"
            :title="t('dialogue.editor.regenerateLine')"
            @click="generate(line.index, true)"
          >
            <LoadingSpinner v-if="generatingLine === line.index" size="sm" />
            <RefreshCw v-else class="h-4 w-4" aria-hidden="true" />
          </button>
        </li>
      </ol>
    </div>

    <!-- Live preview -->
    <details v-if="previewView" class="dialogue-editor__preview" open>
      <summary class="text-sm font-medium text-gray-700">
        {{ t('dialogue.editor.preview') }}
      </summary>
      <LingoDialogueCard
        mode="preview"
        :item-id="itemId"
        :preview-view="previewView"
        :audio-version="audioVersion"
      />
    </details>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw, Volume2 } from '@lucide/vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import { generateDialogueAudio, getDialogue, previewDialogue } from '@/api'
import { useDialogueAudio } from '@/composables/useDialogueAudio'
import LingoDialogueCard from './LingoDialogueCard.vue'
import {
  DIALOGUE_VOICES,
  NARRATOR,
  type Dialogue,
  type DialogueLine,
  type DialogueView,
} from './types'

const props = defineProps<{ modelValue: string; itemId?: number | null }>()
const emit = defineEmits<{ 'update:modelValue': [value: string]; valid: [ok: boolean] }>()
const { t } = useI18n()

const textareaId = `dialogue-script-${Math.random().toString(36).slice(2, 8)}`
const showHelp = ref(false)
const parsing = ref(false)
const parsed = ref<{ dialogue: Dialogue; voices: Record<string, string> } | null>(null)
const saved = ref<DialogueView | null>(null)
const voiceMap = reactive<Record<string, string>>({})
const generating = ref(false)
const generatingLine = ref<number | null>(null)
const audioError = ref('')
const audioVersion = ref(0)
const audio = useDialogueAudio(() => props.itemId)

const errors = computed(() => parsed.value?.dialogue.errors ?? [])
const speakers = computed(() => Object.keys(parsed.value?.voices ?? {}))
const dirty = computed(() => !!saved.value && savedText.value !== props.modelValue)
const savedText = ref('')
const canGenerate = computed(() => !!props.itemId && !dirty.value && !errors.value.length)

function voiceForLine(line: DialogueLine) {
  return line.voice_override || voiceMap[line.speaker] || parsed.value?.voices[line.speaker] || ''
}
function lineHasAudio(index: number) {
  return !dirty.value && !!saved.value?.lines.find((l) => l.index === index)?.has_audio
}

const previewView = computed<DialogueView | null>(() => {
  if (!parsed.value) return null
  return {
    item_id: props.itemId ?? null,
    collection_id: null,
    is_owner: true,
    dialogue: parsed.value.dialogue,
    voices: { ...voiceMap },
    lines: parsed.value.dialogue.lines.map((l) => ({
      index: l.index,
      voice: voiceForLine(l),
      has_audio: lineHasAudio(l.index),
    })),
    bank_chips: dirty.value ? [] : (saved.value?.bank_chips ?? []),
    notes: '',
  }
})

let timer: ReturnType<typeof setTimeout> | null = null
let req = 0
async function runPreview() {
  const mine = ++req
  parsing.value = true
  try {
    const res = await previewDialogue({ text: props.modelValue, item_id: props.itemId ?? null })
    if (mine !== req) return
    parsed.value = res.data
    for (const [sp, v] of Object.entries(res.data.voices as Record<string, string>)) {
      if (!voiceMap[sp]) voiceMap[sp] = v
    }
    emit('valid', !res.data.dialogue.errors.length && res.data.dialogue.lines.length > 0)
  } catch {
    if (mine === req) emit('valid', false)
  } finally {
    if (mine === req) parsing.value = false
  }
}

watch(
  () => props.modelValue,
  () => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(runPreview, 400)
  },
  { immediate: true }
)

/** Saved card text comes back HTML-escaped (`&gt;`); show the raw markup in the editor. */
function decodeEntities(text: string): string {
  if (!/&(?:#\d+|#x[0-9a-f]+|[a-z]+);/i.test(text)) return text
  const el = document.createElement('textarea')
  el.innerHTML = text
  return el.value
}
const decoded = decodeEntities(props.modelValue)
if (decoded !== props.modelValue) emit('update:modelValue', decoded)

async function loadSaved() {
  if (!props.itemId) return
  try {
    const res = await getDialogue(props.itemId)
    saved.value = res.data
    savedText.value = decodeEntities(props.modelValue)
    Object.assign(voiceMap, res.data.voices)
  } catch {
    saved.value = null
  }
}
watch(() => props.itemId, loadSaved, { immediate: true })

async function generate(line: number | null, force: boolean) {
  if (!props.itemId) return
  generating.value = true
  generatingLine.value = line
  audioError.value = ''
  try {
    const res = await generateDialogueAudio(props.itemId, { line, force, voices: { ...voiceMap } })
    saved.value = res.data
    audioVersion.value += 1
    audio.clear()
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    audioError.value =
      status === 429 ? t('dialogue.editor.rateLimited') : t('dialogue.editor.generateFailed')
  } finally {
    generating.value = false
    generatingLine.value = null
  }
}

onBeforeUnmount(() => {
  if (timer) clearTimeout(timer)
})
</script>

<style scoped>
.dialogue-editor {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
.dialogue-editor__textarea {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.85rem;
}
.dialogue-editor__help {
  white-space: pre-wrap;
  font-size: 0.75rem;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  padding: 0.5rem;
}
.dialogue-editor__errors {
  font-size: 0.8rem;
  color: #b42323;
  list-style: disc;
  padding-inline-start: 1.2rem;
}
.dialogue-editor__voices {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 1rem;
}
.dialogue-editor__voice {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.85rem;
}
.dialogue-editor__line-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.dialogue-editor__line {
  display: grid;
  grid-template-columns: 5rem 1fr auto auto auto;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.82rem;
  border-bottom: 1px solid #f3f4f6;
  padding: 0.15rem 0;
}
.dialogue-editor__line-speaker {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
}
.dialogue-editor__line-text {
  overflow-wrap: anywhere;
}
.dialogue-editor__line-voice {
  color: #6b7280;
  font-size: 0.75rem;
}
.dialogue-editor__icon {
  display: inline-flex;
  width: 1.75rem;
  height: 1.75rem;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  color: #1cb0f6;
}
.dialogue-editor__icon:disabled {
  color: #d1d5db;
}
.dialogue-editor__preview {
  border-top: 1px solid #e5e7eb;
  padding-top: 0.5rem;
}
</style>
