<template>
  <h1 class="page-section-title">{{ t('bulkImport.title') }}</h1>

  <div class="flex justify-between my-4">
    <RouterLink to="/bulk-import/clients" class="ui-btn--accent-purple">
      {{ t('bulkImport.viewPastImportsLink') }}
    </RouterLink>
  </div>

  <div class="bg-white shadow rounded-lg p-4 sm:p-6">
    <div class="mb-6">
      <label class="block text-base sm:text-sm font-medium text-gray-700 mb-2">
        {{ t('bulkImport.uploadCsvLabel') }}
      </label>
      <div
        ref="dropZoneRef"
        class="mt-1 flex justify-center px-3 sm:px-6 pt-4 sm:pt-5 pb-4 sm:pb-6 border-2 border-dashed rounded-md transition-colors"
        :class="{
          'border-blue-400 bg-blue-50': isOverDropZone,
          'border-gray-300': !isOverDropZone,
        }"
      >
        <div class="space-y-1 text-center">
          <ImagePlus class="mx-auto h-12 w-12 text-gray-300" :stroke-width="1" />
          <div class="flex justify-center text-sm text-gray-600">
            <label for="file-upload" class="file-input-label">
              <span>{{ t('bulkImport.uploadFile') }}</span>
              <input
                id="file-upload"
                name="file-upload"
                type="file"
                class="sr-only"
                accept=".csv"
                @change="handleFileUpload"
              />
            </label>
            <p class="pl-1">{{ t('bulkImport.dragAndDrop') }}</p>
          </div>

          <div v-if="csvFile" class="mt-2 text-sm text-gray-600">
            <div class="flex items-center justify-center space-x-2">
              <span class="truncate max-w-[200px]">{{ csvFile.name }}</span>
              <button type="button" class="text-red-500 hover:text-red-700" @click="clearFile">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                    clip-rule="evenodd"
                  />
                </svg>
              </button>
            </div>
          </div>

          <p class="text-xs text-gray-500 text-left space-y-1 mt-3">
            <span class="font-medium block">{{ t('bulkImport.csvFormat.title') }}</span>
            <span class="block">{{ t('bulkImport.csvFormat.lineDesc') }}</span>
            <span class="block">{{ t('bulkImport.csvFormat.glossDesc') }}</span>
            <span class="block">{{ t('bulkImport.csvFormat.meaningDesc') }}</span>
            <span class="block">{{ t('bulkImport.csvFormat.example') }}</span>
            <code class="block bg-gray-50 p-2 rounded text-[11px] break-all">
              bajra,$x_1$ runs,Describes fast or slow running,jogging.;slow run,sprint;fast run
            </code>
          </p>
        </div>
      </div>
    </div>

    <div class="mb-6">
      <label for="language" class="block text-sm font-medium text-gray-700 mb-2">
        {{ t('bulkImport.targetLanguageLabel') }}
      </label>
      <select
        id="language"
        v-model="selectedLanguage"
        class="input-field w-full h-8"
        :disabled="isLoading"
      >
        <option value="">{{ t('bulkImport.selectLanguagePlaceholder') }}</option>

        <option v-for="lang in languages" :key="lang.id" :value="lang.id">
          {{ lang.real_name }}
        </option>
      </select>
    </div>

    <div class="flex flex-col sm:flex-row justify-end gap-2 mt-4 sm:mt-0">
      <button
        type="button"
        class="ui-btn--create w-full sm:w-auto order-1"
        :disabled="!canSubmit || isLoading || isCancelling"
        @click="submitImport"
      >
        <span v-if="isLoading"> {{ t('bulkImport.processing') }} </span>
        <span v-else> {{ t('bulkImport.importButton') }} </span>
      </button>
      <button
        v-if="importProcessId"
        type="button"
        class="ui-btn--neutral w-full sm:w-auto order-2"
        :disabled="isCancelling"
        @click="cancelJob"
      >
        <span v-if="isCancelling"> {{ t('bulkImport.cancelling') }} </span>
        <span v-else> {{ t('bulkImport.cancelButton') }} </span>
      </button>
    </div>

    <div v-if="storedClientId" class="my-2">
      <div class="bg-blue-50 border-l-4 border-blue-400 p-3 sm:p-4">
        <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
          <div class="space-y-2">
            <p class="text-sm text-blue-700">
              <span class="block sm:inline">{{ t('bulkImport.clientIdLabel') }}</span>
              <span class="flex items-center gap-2 mt-1 sm:mt-0">
                <strong class="break-all text-xs sm:text-sm font-mono">{{ storedClientId }}</strong>
                <ClipboardButton
                  :content="storedClientId"
                  :title="t('bulkImport.copyClientIdTitle')"
                />
              </span>
            </p>

            <p class="text-xs text-blue-600">{{ t('bulkImport.saveIdNote') }}</p>
          </div>

          <div class="flex items-center">
            <button
              class="ui-btn--delete w-full sm:w-auto"
              :disabled="isDeleting"
              @click="deleteByClientId"
            >
              <span v-if="isDeleting">{{ t('bulkImport.deleting') }}</span>
              <span v-else>{{ t('bulkImport.deleteDefinitionsButton') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="!storedClientId" class="mb-6 mt-6">
      <div class="bg-gray-100 p-3 sm:p-4 rounded-lg border-2 border-gray-200 shadow-sm space-y-3">
        <div class="flex justify-between items-center">
          <label class="block text-base sm:text-sm font-semibold text-gray-700">
            {{ t('bulkImport.deleteByIdTitle') }}
          </label>
        </div>

        <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 w-full">
          <input
            v-model="inputClientId"
            type="text"
            :placeholder="t('bulkImport.pasteClientIdPlaceholder')"
            class="input-field flex-1 text-xs sm:text-sm font-mono"
          />
          <button
            class="ui-btn--delete w-full sm:w-auto"
            :disabled="!inputClientId || isDeleting"
            @click="deleteByClientId"
          >
            <span v-if="isDeleting">{{ t('bulkImport.deleting') }}</span>
            <span v-else>{{ t('bulkImport.deleteButton') }}</span>
          </button>
        </div>
      </div>
    </div>

    <div class="mb-6 space-y-4">
      <div
        v-if="statusMessage"
        class="border-l-4 p-4"
        :class="{
          'bg-green-50 border-green-400 text-green-700': statusType === 'success',
          'bg-red-50 border-red-400 text-red-700': statusType === 'error',
        }"
      >
        <p class="text-sm">{{ statusMessage }}</p>
      </div>

      <div
        v-if="logs.length"
        ref="logContainerRef"
        class="border rounded-lg p-3 sm:p-4 bg-gray-50 max-h-48 overflow-y-auto text-sm sm:text-base"
      >
        <div
          v-for="(log, index) in logs.slice().reverse()"
          :key="index"
          class="text-sm mb-2 last:mb-0"
          :class="{
            'text-green-600': log.type === 'success',
            'text-blue-600': log.type === 'info',
            'text-red-600': log.type === 'error',
          }"
        >
          <span class="font-medium">{{ log.current }}. </span>
          <span class="font-medium">Processed</span>
          <span v-if="log.word" class="font-medium">: </span>
          <span
            v-if="log.word"
            class="font-medium text-slate-600 p-1 border border-slate-300 rounded"
            >{{ log.word }}</span
          >
          <span v-if="log.details" class="text-gray-600 text-xs block mt-1">{{ log.details }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDropZone } from '@vueuse/core'
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { ImagePlus } from 'lucide-vue-next'

import {
  cancelBulkImport,
  deleteBulkDefinitions,
  getApiBaseUrl,
  getAuthHeaders,
  getLanguages,
} from '../api'
import ClipboardButton from '@/components/ClipboardButton.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useError } from '@/composables/useError'

const { t } = useI18n()
const { showError } = useError()

useSeoHead({ title: t('bulkImport.title'), robots: 'noindex, nofollow' })

const selectedLanguage = ref('')
const csvFile = ref(null)

const languages = ref([])
const isLoading = ref(false)
const isCancelling = ref(false)
const importProcessId = ref(
  typeof window === 'undefined' ? '' : localStorage.getItem('lastImportProcessId') || ''
)
const statusMessage = ref('')
const statusType = ref('')
const dropZoneRef = ref()

const { isOverDropZone } = useDropZone(dropZoneRef, (files) => {
  if (files && files.length > 0) {
    csvFile.value = files[0]
  }
})

const loadLanguages = async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch {
    showError(t('bulkImport.status.loadLanguagesError'))
  }
}

const handleFileUpload = (event) => {
  const input = event.target
  const files = input.files
  if (files && files.length > 0) {
    csvFile.value = files[0]
  }
}

const clearFile = () => {
  csvFile.value = null
  const fileInput = document.getElementById('file-upload') as HTMLInputElement | null
  if (fileInput) {
    fileInput.value = ''
  }
}

const logs = ref([])
const storedClientId = ref(
  typeof window === 'undefined' ? '' : localStorage.getItem('lastImportClientId') || ''
)
const inputClientId = ref('')
const isDeleting = ref(false)
const MAX_LOG_LINES = 200
const abortController = ref(null)
const logContainerRef = ref(null)

watch(
  logs,
  async () => {
    await nextTick()
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    }
  },
  { deep: true }
)

watch(storedClientId, (newVal) => {
  if (newVal) {
    inputClientId.value = newVal
  }
})

const submitImport = async () => {
  if (typeof window === 'undefined') return

  if (!canSubmit.value) return

  isLoading.value = true
  statusMessage.value = ''
  logs.value = []
  importProcessId.value = ''
  storedClientId.value = ''
  localStorage.removeItem('lastImportProcessId')
  localStorage.removeItem('lastImportClientId')
  abortController.value = new AbortController()

  try {
    const fileContent = await new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = (e) => e?.target !== null && resolve(e.target.result)
      reader.onerror = (e) => reject(e)
      if (csvFile.value !== null) reader.readAsText(csvFile.value)
    })

    const url = `${getApiBaseUrl()}/jbovlaste/bulk-import`
    const headers = {
      'Content-Type': 'application/json',
      ...getAuthHeaders(),
    }

    const response = await fetch(url, {
      method: 'POST',
      headers: headers,
      body: JSON.stringify({
        lang_id: parseInt(selectedLanguage.value),
        csv: fileContent,
      }),
      signal: abortController.value.signal,
    })

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(`HTTP error! status: ${response.status}, message: ${errorText}`)
    }

    if (!response.body) {
      throw new Error('Response body is null')
    }

    const reader = response.body.pipeThrough(new TextDecoderStream()).getReader()

    while (abortController.value && !abortController.value.signal.aborted) {
      const { value, done } = await reader.read()
      if (done) break

      const values = value
        .split(/\n/)
        .filter(Boolean)
        .map((el) => el.replace(/^data: */, ''))

      for (const rawValue of values) {
        try {
          if (rawValue.trim().startsWith('{')) {
            const event = JSON.parse(rawValue)

            if (event.type === 'client_id') {
              importProcessId.value = event.client_id
              localStorage.setItem('lastImportProcessId', event.client_id)
              storedClientId.value = event.client_id
              logs.value.push({
                type: 'info',
                details: t('bulkImport.status.importStarted'),
                current: 0,
                word: '',
              })
            } else if (event.type === 'start') {
              logs.value.push({
                type: 'info',
                details: `Starting import of ${event.total} records.`,
                current: 0,
                word: '',
              })
            } else if (event.type === 'progress') {
              logs.value.push({
                type: event.success ? 'success' : 'error',
                details: event.success
                  ? `Imported successfully. (${event.success_count}✓ ${event.error_count}✗)`
                  : `${t('bulkImport.status.importError')} ${event.error} (${event.success_count}✓ ${event.error_count}✗)`,
                current: event.current,
                word: event.word,
              })

              if (logs.value.length > MAX_LOG_LINES) {
                logs.value.shift()
              }
            } else if (event.type === 'complete') {
              setStatus(event.message, event.success ? 'success' : 'error')
              if (event.client_id) {
                storedClientId.value = event.client_id
                localStorage.setItem('lastImportClientId', event.client_id)
              }
              logs.value.push({
                type: event.success ? 'success' : 'error',
                details: t('bulkImport.status.importFinished', {
                  success_count: event.success_count,
                  error_count: event.error_count,
                }),
                current: event.total_processed,
                word: t('bulkImport.status.endMarker'),
              })
              break
            } else if (event.type === 'error') {
              setStatus(`${t('bulkImport.status.importFailed')}: ${event.error}`, 'error')
              logs.value.push({
                type: 'error',
                details: `${t('bulkImport.status.fatalError')} ${event.error}`,
              })
              break
            }
          }
        } catch (error) {
          logs.value.push({
            type: 'error',
            message: 'Error processing SSE event',
            details: `Raw data: ${rawValue}. Error: ${error instanceof Error ? error.message : 'Unknown error'}`,
          })
        }
      }
    }

    if (!statusMessage.value && !abortController.value?.signal.aborted) {
      setStatus(t('bulkImport.status.unexpectedEnd'), 'error')
      logs.value.push({ type: 'error', details: t('bulkImport.status.connectionClosed') })
    }
  } catch (error) {
    logs.value.push({
      type: 'error',
      message: t('bulkImport.status.importFailed'),
      details: error instanceof Error ? error.message : 'Unknown error',
    })
    setStatus(t('bulkImport.status.importFailed'), 'error')
  } finally {
    isLoading.value = false
    isCancelling.value = false
    abortController.value = null
    importProcessId.value = ''
    localStorage.removeItem('lastImportProcessId')
  }
}

const setStatus = (message, type = 'success') => {
  statusMessage.value = message
  statusType.value = type
}

const canSubmit = computed(() => {
  return selectedLanguage.value && csvFile.value && !isLoading.value
})

onMounted(() => {
  void loadLanguages()
  const savedProcessId = localStorage.getItem('lastImportProcessId')
  if (savedProcessId) {
    importProcessId.value = savedProcessId
    logs.value.push({
      type: 'info',
      details: t('bulkImport.status.foundExistingProcess'),
      current: 0,
      word: '',
    })
  }
})

const cancelJob = async () => {
  if (!importProcessId.value) return

  isCancelling.value = true
  try {
    const response = await cancelBulkImport(importProcessId.value)

    if (!(response.status >= 200 && response.status < 300)) {
      throw new Error(`Failed to cancel job: ${response.status} ${response.statusText}`)
    }

    logs.value.push({
      type: 'info',
      details: t('bulkImport.status.cancellationRequested'),
      current: 0,
      word: '',
    })
    statusMessage.value = t('bulkImport.status.cancellationRequested')
  } catch (error) {
    logs.value.push({
      type: 'error',
      details: error instanceof Error ? error.message : 'Unknown error',
      current: 0,
      word: '',
    })
    statusMessage.value = t('bulkImport.status.cancelFailed')
  } finally {
    isCancelling.value = false
  }
}

const deleteByClientId = async () => {
  if (typeof window === 'undefined') return

  const clientIdToDelete = inputClientId.value || storedClientId.value
  if (!clientIdToDelete) return

  isDeleting.value = true
  try {
    const response = await deleteBulkDefinitions(clientIdToDelete)

    if (response.status !== 200) {
      throw new Error(await response.statusText)
    }

    const result = await response.data
    logs.value.push({
      type: 'success',
      details: t('bulkImport.status.deletedDefinitions', {
        deleted: result.deleted?.length || 0,
        skipped: result.skipped?.length || 0,
      }),
      current: 0,
      word: '',
    })

    if (clientIdToDelete === storedClientId.value) {
      storedClientId.value = ''
      localStorage.removeItem('lastImportClientId')
    }
    inputClientId.value = ''
  } catch (error) {
    logs.value.push({
      type: 'error',
      details: error instanceof Error ? error.message : 'Unknown error',
      current: 0,
      word: '',
    })
  } finally {
    isDeleting.value = false
    inputClientId.value = ''
    storedClientId.value = ''
    localStorage.removeItem('lastImportClientId')
  }
}

onBeforeUnmount(() => {
  if (abortController.value) {
    abortController.value.abort()
  }
})
</script>
