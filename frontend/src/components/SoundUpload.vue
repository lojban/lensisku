<template>

  <div>

    <div class="flex items-center justify-between">
       <label class="block text-sm font-medium text-blue-700"
        > {{ label || t('soundUpload.sound') }} </label
      > <button
        v-if="modelValue || loadedSound"
        type="button"
        class="text-sm text-red-600 hover:text-red-700"
        @click="handleRemove"
      >
         {{ t('soundUpload.removeSound') }} </button
      > <span v-else-if="note" class="text-xs text-gray-500"> {{ note }} </span>
    </div>
     <!-- Sound Preview -->
    <div
      v-if="modelValue || loadedSound"
      class="relative flex flex-col items-center mt-2 p-4 border rounded-lg bg-gray-50"
    >

      <div class="flex items-center gap-4 mb-2">
         <Volume2 class="h-8 w-8 text-blue-500" /> <span class="text-sm text-gray-600">{{
          fileName || 'Custom Audio'
        }}</span
        >
      </div>
       <audio controls :src="audioUrl" class="w-full h-10" />
    </div>
     <!-- No sound: choose Upload or Record -->
    <div v-if="!modelValue && !loadedSound" class="mt-2 space-y-3">
       <!-- Tabs: Upload | Record | Generate (voices: keep in sync with src/utils/kitten_tts.rs voice_aliases) -->
      <div
        class="flex flex-wrap rounded-lg border border-gray-200 p-1 bg-gray-50 gap-1"
        role="tablist"
        aria-label="Add sound by upload, record, or generate"
      >
         <button
          type="button"
          role="tab"
          :aria-selected="inputMode === 'upload'"
          :class="[
            'flex-1 min-w-[5.5rem] flex items-center justify-center gap-1.5 py-2.5 px-2 rounded-md text-sm font-medium transition-colors',
            inputMode === 'upload'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900',
          ]"
          @click="inputMode = 'upload'"
        >
           <Upload class="h-4 w-4 shrink-0" /> {{ t('soundUpload.uploadTab') }} </button
        > <button
          type="button"
          role="tab"
          :aria-selected="inputMode === 'record'"
          :class="[
            'flex-1 min-w-[5.5rem] flex items-center justify-center gap-1.5 py-2.5 px-2 rounded-md text-sm font-medium transition-colors',
            inputMode === 'record'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900',
          ]"
          @click="inputMode = 'record'; recordingError = ''"
        >
           <Mic class="h-4 w-4 shrink-0" /> {{ t('soundUpload.recordTab') }} </button
        > <button
          type="button"
          role="tab"
          :aria-selected="inputMode === 'generate'"
          :class="[
            'flex-1 min-w-[5.5rem] flex items-center justify-center gap-1.5 py-2.5 px-2 rounded-md text-sm font-medium transition-colors',
            inputMode === 'generate'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900',
          ]"
          @click="onSelectGenerateTab"
        >
           <Sparkles class="h-4 w-4 shrink-0" /> {{ t('soundUpload.generateTab') }} </button
        >
      </div>
       <!-- Upload panel -->
      <div
        v-show="inputMode === 'upload'"
        ref="dropZoneRef"
        class="flex justify-center px-6 pt-5 pb-6 border-2 border-dashed rounded-lg transition-colors"
        :class="{
          'border-blue-400 bg-blue-50': isOverDropZone,
          'border-gray-300': !isOverDropZone,
        }"
      >

        <div class="space-y-1 text-center">
           <Upload class="mx-auto h-12 w-12 text-gray-300" :stroke-width="1" />
          <div class="flex flex-wrap justify-center gap-x-1 text-sm text-gray-600">
             <label
              class="relative cursor-pointer rounded-md font-medium text-blue-600 hover:text-blue-500"
              > <span>{{ t('soundUpload.uploadPrompt') }}</span
              > <input
                type="file"
                class="sr-only"
                accept="audio/mpeg,audio/mp3,audio/ogg,audio/webm"
                @change="handleFileSelect"
              /> </label
            >
            <p>{{ t('soundUpload.dragDrop') }}</p>

          </div>

          <p class="text-xs text-gray-500"> {{ t('soundUpload.fileTypes') }} </p>

        </div>

      </div>
       <!-- Record panel -->
      <div
        v-show="inputMode === 'record'"
        class="border border-gray-200 rounded-lg p-4 bg-gray-50/50"
      >
         <!-- Permission / not started --> <template v-if="!isRecording && !recordedBlob"
          >
          <p class="text-sm text-gray-600 mb-3"> {{ t('soundUpload.recordHint') }} </p>
           <div class="flex w-full justify-center">
             <Button
              variant="danger-rose"
              size="lg"
              class="w-full sm:w-auto"
              :loading="isRequestingMic"
              :disabled="isRequestingMic"
              @click="startRecording"
            >
               <template #icon>
                <Mic v-if="!isRequestingMic" class="h-5 w-5 shrink-0" />
              </template>
              {{
                isRequestingMic ? t('soundUpload.requestingMic') : t('soundUpload.startRecording')
              }} </Button
            >
          </div>
          <p v-if="recordingError" class="mt-2 text-sm text-red-600" role="alert">
             {{ recordingError }}
          </p>
           </template
        > <!-- Recording in progress --> <template v-else-if="isRecording"
          >
          <div class="flex items-center justify-center gap-3 py-2">
             <span class="relative flex h-3 w-3" aria-hidden="true"
              > <span
                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"
              /> <span class="relative inline-flex rounded-full h-3 w-3 bg-red-500" /> </span
            > <span class="text-sm font-medium text-gray-700 tabular-nums"
              > {{ t('soundUpload.recordingTime', { seconds: recordingSeconds }) }} </span
            >
          </div>
           <div class="mt-3 flex w-full justify-center">
             <Button variant="cancel" size="lg" class="w-full sm:w-auto" @click="stopRecording">
               <template #icon>
                <Square class="h-5 w-5 shrink-0" />
              </template>
              {{ t('soundUpload.stopRecording') }} </Button
            >
          </div> </template
        > <!-- Recorded preview: use or re-record --> <template v-else-if="recordedBlob"
          >
          <div class="space-y-3">
             <audio :src="recordedPreviewUrl" controls class="w-full h-10" />
            <div
              class="flex w-full flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-center sm:gap-3"
            >
               <Button
                variant="insert"
                size="lg"
                class="w-full sm:w-auto"
                @click="useRecording"
              >
                 <template #icon>
                  <Check class="h-5 w-5 shrink-0" aria-hidden="true" />
                </template>
                {{ t('soundUpload.useRecording') }} </Button
              > <Button
                variant="cancel"
                size="lg"
                class="w-full sm:w-auto"
                @click="discardRecording"
              >
                 <template #icon>
                  <RotateCcw class="h-5 w-5 shrink-0" aria-hidden="true" />
                </template>
                {{ t('soundUpload.reRecord') }} </Button
              >
            </div>

          </div>
           </template
        >
      </div>
       <!-- Generate panel (Kitten TTS) -->
      <div
        v-show="inputMode === 'generate'"
        class="border border-gray-200 rounded-lg p-4 bg-gray-50/50 space-y-3"
      >
         <p class="text-sm text-gray-600"> {{ t('soundUpload.generateHint') }} </p>
        <div>
           <label class="block text-sm font-medium text-gray-700 mb-1"
            >{{ t('soundUpload.generateTextLabel') }}</label
          > <textarea
            v-model="generateText"
            rows="3"
            class="textarea-field w-full text-sm"
            :disabled="isGenerating"
          />
        </div>
        <div>
           <label class="block text-sm font-medium text-gray-700 mb-1"
            >{{ t('soundUpload.voiceLabel') }}</label
          > <select
            v-model="selectedVoice"
            class="input-field w-full text-sm"
            :disabled="isGenerating"
          >

            <option v-for="v in KITTEN_VOICES" :key="v" :value="v">
               {{ v }}
            </option>
             </select
          >
        </div>
         <p v-if="generateError" class="text-sm text-red-600" role="alert">
           {{ generateError }}
        </p>
         <div class="flex w-full justify-center">
           <Button
            variant="insert"
            size="lg"
            class="w-full sm:w-auto"
            :loading="isGenerating"
            :disabled="isGenerating || !generateText.trim()"
            @click="runKittenGenerate"
          >
             <template #icon>
              <Sparkles v-if="!isGenerating" class="h-5 w-5 shrink-0" />
            </template>
            {{
              isGenerating ? t('soundUpload.generating') : t('soundUpload.generateButton')
            }} </Button
          >
        </div>
      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { Check, Mic, RotateCcw, Square, Sparkles, Upload, Volume2 } from 'lucide-vue-next'
import { useDropZone } from '@vueuse/core'
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

import { useError } from '../composables/useError'
import { Button } from '@packages/ui'
import { generateKittenTts, getItemSoundBlob } from '@/api'
import { getApiErrorMessage } from '@/utils/apiError'

const { showError, clearError } = useError()
const props = defineProps({
  modelValue: {
    type: Object,
    default: null,
  },
  collectionId: {
    type: Number,
    default: null,
  },
  itemId: {
    type: Number,
    default: null,
  },
  label: {
    type: String,
    default: '',
  },
  note: {
    type: String,
    default: '',
  },
  hasExistingSound: {
    type: Boolean,
    default: false,
  },
  /** Prefills the Generate tab when empty (e.g. card front / word). */
  defaultTtsText: {
    type: String,
    default: '',
  },
})

/** Display names accepted by the server (`src/utils/kitten_tts.rs` voice_aliases). */
const KITTEN_VOICES = ['Bella', 'Jasper', 'Luna', 'Bruno', 'Rosie', 'Hugo', 'Kiki', 'Leo']

const emit = defineEmits(['update:modelValue', 'sound-loaded', 'remove-sound'])

const previewUrl = ref('')
const loadedSound = ref(null)
const isLoading = ref(false)
const dropZoneRef = ref(null)
const fileName = ref('')

// Record-from-mic state
const inputMode = ref('upload')
const generateText = ref('')
const selectedVoice = ref('Bruno')
const isGenerating = ref(false)
const generateError = ref('')
const isRecording = ref(false)
const isRequestingMic = ref(false)
const recordingError = ref('')
const recordingSeconds = ref(0)
const recordedBlob = ref(null)
const recordedPreviewUrl = ref('')
let mediaStream = null
let mediaRecorder = null
let recordingTimer = null

function getSupportedMimeType() {
  const types = ['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus', 'audio/mp4']
  for (const type of types) {
    if (MediaRecorder.isTypeSupported(type)) return type
  }
  return ''
}

function startRecording() {
  recordingError.value = ''
  isRequestingMic.value = true
  navigator.mediaDevices
    .getUserMedia({ audio: true })
    .then((stream) => {
      isRequestingMic.value = false
      mediaStream = stream
      const mimeType = getSupportedMimeType()
      const options = mimeType ? { mimeType } : {}
      mediaRecorder = new MediaRecorder(stream, options)
      const chunks = []

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunks.push(e.data)
      }
      mediaRecorder.onstop = () => {
        if (chunks.length) {
          const mime = mediaRecorder.mimeType || 'audio/webm'
          recordedBlob.value = new Blob(chunks, { type: mime })
          recordedPreviewUrl.value = URL.createObjectURL(recordedBlob.value)
        }
        if (mediaStream) {
          mediaStream.getTracks().forEach((t) => t.stop())
          mediaStream = null
        }
        mediaRecorder = null
      }

      mediaRecorder.start()
      isRecording.value = true
      recordingSeconds.value = 0
      recordingTimer = setInterval(() => {
        recordingSeconds.value += 1
      }, 1000)
    })
    .catch((err) => {
      isRequestingMic.value = false
      console.error('Microphone access error:', err)
      if (err.name === 'NotAllowedError' || err.name === 'PermissionDeniedError') {
        recordingError.value = t('soundUpload.micPermissionDenied')
      } else {
        recordingError.value = t('soundUpload.micError')
      }
    })
}

function stopRecording() {
  if (!mediaRecorder || mediaRecorder.state === 'inactive') return
  mediaRecorder.stop()
  isRecording.value = false
  if (recordingTimer) {
    clearInterval(recordingTimer)
    recordingTimer = null
  }
}

function syncGenerateTextFromDefault() {
  const d = props.defaultTtsText?.trim()
  if (d && !generateText.value.trim()) {
    generateText.value = d
  }
}

function onSelectGenerateTab() {
  inputMode.value = 'generate'
  recordingError.value = ''
  syncGenerateTextFromDefault()
}

async function extractAxiosErrorMessage(err) {
  const res = err?.response
  if (res?.data instanceof Blob) {
    try {
      const txt = await res.data.text()
      const j = JSON.parse(txt)
      if (typeof j.error === 'string') return j.error
      return txt
    } catch {
      return t('soundUpload.generateFailed')
    }
  }
  return (
    getApiErrorMessage(err) ||
    (typeof err?.message === 'string' ? err.message : '') ||
    t('soundUpload.generateFailed')
  )
}

async function runKittenGenerate() {
  generateError.value = ''
  const text = generateText.value.trim()
  if (!text) return
  isGenerating.value = true
  try {
    const response = await generateKittenTts({
      text,
      voice: selectedVoice.value,
      speed: 1.0,
    })
    const blob = response.data
    if (!(blob instanceof Blob)) {
      throw new Error(t('soundUpload.generateFailed'))
    }
    const base64 = await blobToBase64(blob)
    fileName.value = t('soundUpload.generatedAudio')
    emit('update:modelValue', {
      data: base64,
      mime_type: blob.type || 'audio/ogg',
    })
    clearError()
  } catch (e) {
    const status = e?.response?.status
    if (status === 429) {
      generateError.value = t('soundUpload.rateLimitExceeded')
    } else {
      generateError.value = await extractAxiosErrorMessage(e)
    }
    showError(generateError.value)
  } finally {
    isGenerating.value = false
  }
}

function blobToBase64(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const raw = reader.result
      const dataUrl = typeof raw === 'string' ? raw : ''
      const base64 = dataUrl.split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })
}

async function useRecording() {
  if (!recordedBlob.value) return
  try {
    const base64 = await blobToBase64(recordedBlob.value)
    const soundObj = {
      data: base64,
      mime_type: recordedBlob.value.type,
    }
    fileName.value = t('soundUpload.recordedAudio')
    emit('update:modelValue', soundObj)
    discardRecording()
  } catch (e) {
    showError(e.message || 'Failed to use recording')
  }
}

function discardRecording() {
  if (recordedPreviewUrl.value) {
    URL.revokeObjectURL(recordedPreviewUrl.value)
    recordedPreviewUrl.value = ''
  }
  recordedBlob.value = null
  recordingSeconds.value = 0
}

const audioUrl = computed(() => {
  if (props.modelValue?.dataUri) return props.modelValue.dataUri
  if (props.modelValue?.data)
    return `data:${props.modelValue.mime_type};base64,${props.modelValue.data}`
  return previewUrl.value
})

const { isOverDropZone } = useDropZone(dropZoneRef, (files) => {
  if (files && files.length > 0) {
    processFile(files[0])
  }
})

const clearSound = () => {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = ''
  }
  loadedSound.value = null
  fileName.value = ''
  discardRecording()
  generateError.value = ''
}

const handleRemove = () => {
  clearSound()
  emit('update:modelValue', null)
  emit('remove-sound')
}

const loadExistingSound = async () => {
  if (!props.hasExistingSound || !props.collectionId || !props.itemId || isLoading.value) return

  try {
    isLoading.value = true
    clearSound()
    clearError()

    const response = await getItemSoundBlob(props.collectionId, props.itemId)
    const blob = response.data
    previewUrl.value = URL.createObjectURL(blob)
    fileName.value = 'Existing Audio'

    const reader = new FileReader()
    reader.onload = (e) => {
      const raw = e.target?.result
      const dataUrl = typeof raw === 'string' ? raw : ''
      const base64String = dataUrl.split(',')[1]
      loadedSound.value = {
        data: base64String,
        mime_type: blob.type,
      }
      emit('sound-loaded', loadedSound.value)
    }
    reader.readAsDataURL(blob)
  } catch (e) {
    if (e.response?.status === 404) return
    console.error('Error loading sound:', e)
    clearSound()
  } finally {
    isLoading.value = false
  }
}

const ALLOWED_AUDIO_TYPES = ['audio/mpeg', 'audio/mp3', 'audio/ogg', 'audio/webm']
const validateFile = (file) => {
  const ok = ALLOWED_AUDIO_TYPES.some((t) => file.type === t || file.type.startsWith(t + ';'))
  if (!ok) {
    throw new Error(t('soundUpload.invalidFileType'))
  }

  if (file.size > 5 * 1024 * 1024) {
    throw new Error('File size exceeds 5MB limit.')
  }
}

const processFile = async (file) => {
  try {
    validateFile(file)
    clearError()
    clearSound()

    previewUrl.value = URL.createObjectURL(file)
    fileName.value = file.name

    const reader = new FileReader()
    reader.onload = (e) => {
      const raw = e.target?.result
      const dataUrl = typeof raw === 'string' ? raw : ''
      const base64String = dataUrl.split(',')[1]
      const soundObj = {
        data: base64String,
        mime_type: file.type,
      }
      emit('update:modelValue', soundObj)
    }
    reader.readAsDataURL(file)
  } catch (e) {
    showError(e.message)
    emit('update:modelValue', null)
  }
}

const handleFileSelect = (event) => {
  const file = event.target.files[0]
  if (file) {
    processFile(file)
  }
}

watch(
  () => props.modelValue,
  (newValue) => {
    if (!newValue && !props.hasExistingSound) {
      clearSound()
    }
  }
)

watch(
  () => props.defaultTtsText,
  () => {
    syncGenerateTextFromDefault()
  },
  { immediate: true }
)

watch(
  () => props.hasExistingSound,
  (newValue) => {
    if (newValue) {
      loadExistingSound()
    } else {
      clearSound()
    }
  },
  { immediate: true }
)

onMounted(() => {
  if (props.hasExistingSound && props.collectionId && props.itemId) {
    loadExistingSound()
  }
})

onUnmounted(() => {
  if (recordingTimer) {
    clearInterval(recordingTimer)
    recordingTimer = null
  }
  if (mediaStream) {
    mediaStream.getTracks().forEach((t) => t.stop())
    mediaStream = null
  }
  if (mediaRecorder && mediaRecorder.state !== 'inactive') {
    mediaRecorder.stop()
  }
  if (recordedPreviewUrl.value) {
    URL.revokeObjectURL(recordedPreviewUrl.value)
  }
})
</script>

