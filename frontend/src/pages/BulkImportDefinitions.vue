<template>

  <h1 class="text-2xl font-bold text-gray-800"> {{ t('bulkImport.title') }} </h1>

  <div class="flex justify-between my-4">
     <RouterLink to="/bulk-import/clients" class="btn-aqua-purple"
      > {{ t('bulkImport.viewPastImportsLink') }} </RouterLink
    >
  </div>

  <div class="bg-white shadow rounded-lg p-4 sm:p-6">
     <!-- File Upload -->
    <div class="mb-6">
       <label class="block text-base sm:text-sm font-medium text-gray-700 mb-2"
        > {{ t('bulkImport.uploadCsvLabel') }} </label
      >
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
             <label
              for="file-upload"
              class="relative cursor-pointer bg-white rounded-md font-medium text-blue-600 hover:text-blue-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-blue-500"
              > <span>{{ t('bulkImport.uploadFile') }}</span
              > <input
                id="file-upload"
                name="file-upload"
                type="file"
                class="sr-only"
                accept=".csv"
                @change="handleFileUpload"
              /> </label
            >
            <p class="pl-1"> {{ t('bulkImport.dragAndDrop') }} </p>

          </div>
           <!-- File name display and clear button -->
          <div v-if="csvFile" class="mt-2 text-sm text-gray-600">

            <div class="flex items-center justify-center space-x-2">
               <span class="truncate max-w-[200px]">{{ csvFile.name }}</span
              > <button type="button" class="text-red-500 hover:text-red-700" @click="clearFile">
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
                   </svg
                > </button
              >
            </div>

          </div>

          <p class="text-xs text-gray-500 text-left space-y-1 mt-3">
             <span class="font-medium block">{{ t('bulkImport.csvFormat.title') }}</span
            > <span class="block">{{ t('bulkImport.csvFormat.lineDesc') }}</span
            > <span class="block">{{ t('bulkImport.csvFormat.glossDesc') }}</span
            > <span class="block">{{ t('bulkImport.csvFormat.meaningDesc') }}</span
            > <span class="block">{{ t('bulkImport.csvFormat.example') }}</span
            > <code class="block bg-gray-50 p-2 rounded text-[11px] break-all"
              > bajra,$x_1$ runs,Describes fast or slow running,jogging.;slow run,sprint;fast run
              </code
            >
          </p>

        </div>

      </div>

    </div>
     <!-- Language Selection -->
    <div class="mb-6">
       <label for="language" class="block text-sm font-medium text-gray-700 mb-2"
        > {{ t('bulkImport.targetLanguageLabel') }} </label
      > <select
        id="language"
        v-model="selectedLanguage"
        class="input-field w-full h-8"
        :disabled="isLoading"
      >

        <option value=""> {{ t('bulkImport.selectLanguagePlaceholder') }} </option>

        <option v-for="lang in languages" :key="lang.id" :value="lang.id">
           {{ lang.real_name }} ({{ lang.english_name }})
        </option>
         </select
      >
    </div>
     <!-- Submit Button -->
    <div class="flex flex-col sm:flex-row justify-end gap-2 mt-4 sm:mt-0">
       <button
        type="button"
        class="btn-aqua-emerald w-full sm:w-auto order-1"
        :disabled="!canSubmit || isLoading || isCancelling"
        @click="submitImport"
      >
         <span v-if="isLoading"> {{ t('bulkImport.processing') }} </span> <span v-else
          > {{ t('bulkImport.importButton') }} </span
        > </button
      > <button
        v-if="importProcessId"
        type="button"
        class="btn-aqua-white w-full sm:w-auto order-2"
        :disabled="isCancelling"
        @click="cancelJob"
      >
         <span v-if="isCancelling"> {{ t('bulkImport.cancelling') }} </span> <span v-else
          > {{ t('bulkImport.cancelButton') }} </span
        > </button
      >
    </div>
     <!-- Client ID and Delete (using storedClientId from completion event) -->
    <div v-if="storedClientId" class="my-2">

      <div class="bg-blue-50 border-l-4 border-blue-400 p-3 sm:p-4">

        <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
           <!-- Left Column -->
          <div class="space-y-2">

            <p class="text-sm text-blue-700">
               <span class="block sm:inline">{{ t('bulkImport.clientIdLabel') }}</span
              > <span class="flex items-center gap-2 mt-1 sm:mt-0"
                > <strong class="break-all text-xs sm:text-sm font-mono">{{
                  storedClientId
                }}</strong
                > <ClipboardButton
                  :content="storedClientId"
                  :title="t('bulkImport.copyClientIdTitle')"
                /> </span
              >
            </p>

            <p class="text-xs text-blue-600"> {{ t('bulkImport.saveIdNote') }} </p>

          </div>
           <!-- Right Column -->
          <div class="flex items-center">
             <button
              class="btn-aqua-red w-full sm:w-auto"
              :disabled="isDeleting"
              @click="deleteByClientId"
            >
               <span v-if="isDeleting">{{ t('bulkImport.deleting') }}</span
              > <span v-else>{{ t('bulkImport.deleteDefinitionsButton') }}</span
              > </button
            >
          </div>

        </div>

      </div>

    </div>
     <!-- Manual Client ID Input -->
    <div v-if="!storedClientId" class="mb-6 mt-6">

      <div class="bg-gray-100 p-3 sm:p-4 rounded-lg border-2 border-gray-200 shadow-sm space-y-3">

        <div class="flex justify-between items-center">
           <label class="block text-base sm:text-sm font-semibold text-gray-700"
            > {{ t('bulkImport.deleteByIdTitle') }} </label
          >
        </div>

        <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 w-full">
           <input
            v-model="inputClientId"
            type="text"
            :placeholder="t('bulkImport.pasteClientIdPlaceholder')"
            class="input-field flex-1 text-xs sm:text-sm font-mono"
          /> <button
            class="btn-aqua-red w-full sm:w-auto"
            :disabled="!inputClientId || isDeleting"
            @click="deleteByClientId"
          >
             <span v-if="isDeleting">{{ t('bulkImport.deleting') }}</span
            > <span v-else>{{ t('bulkImport.deleteButton') }}</span
            > </button
          >
        </div>

      </div>

    </div>
     <!-- Status and Results -->
    <div class="mb-6 space-y-4">
       <!-- Final Status -->
      <div
        v-if="statusMessage"
        class="border-l-4 p-4"
        :class="{
          'bg-green-50 border-green-400 text-green-700': statusType === 'success',
          'bg-red-50 border-red-400 text-red-700': statusType === 'error',
        }"
      >

        <p class="text-sm"> {{ statusMessage }} </p>

      </div>
       <!-- Progress Log -->
      <div
        ref="logContainerRef"
        v-if="logs.length"
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
           <span class="font-medium">{{ log.current }}. </span> <span class="font-medium"
            >Processed</span
          > <span v-if="log.word" class="font-medium">: </span> <span
            v-if="log.word"
            class="font-medium text-slate-600 p-1 border border-slate-300 rounded"
            >{{ log.word }}</span
          > <span v-if="log.details" class="text-gray-600 text-xs block mt-1">{{
            log.details
          }}</span
          >
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

const { t, locale } = useI18n()
const { showError } = useError()

useSeoHead({ title: t('bulkImport.title') })

const selectedLanguage = ref('')
const csvFile = ref(null)

const languages = ref([])
const isLoading = ref(false)
const isCancelling = ref(false)
// Use a more descriptive name, this ID is now for the active process (cancellation) and later deletion
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

// Load available languages
const loadLanguages = async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (error) {
    showError(t('bulkImport.status.loadLanguagesError'))
  }
}

// Handle file upload
const handleFileUpload = (event) => {
  const input = event.target
  const files = input.files
  if (files && files.length > 0) {
    csvFile.value = files[0]
  }
}

// Clear selected file
const clearFile = () => {
  csvFile.value = null
  // Clear the file input value
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

// Watch for changes in logs and scroll to bottom
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

// Sync inputClientId when storedClientId changes
watch(storedClientId, (newVal) => {
  if (newVal) {
    inputClientId.value = newVal
  }
})

// Check for existing import process ID on mount
onMounted(() => {
  if (typeof window === 'undefined') return

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

const submitImport = async () => {
  if (typeof window === 'undefined') return

  if (!canSubmit.value) return

  isLoading.value = true
  statusMessage.value = ''
  logs.value = []
  importProcessId.value = ''
  storedClientId.value = ''
  localStorage.removeItem('lastImportProcessId') // Clear from storage
  localStorage.removeItem('lastImportClientId') // Clear deletion ID from storage
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
        .map((el) => el.replace(/^data: */, '')) // Remove "data: " prefix

      for (const rawValue of values) {
        console.log('Raw SSE received:', rawValue) // Log raw value
        try {
          // Ensure we only parse if it looks like JSON
          if (rawValue.trim().startsWith('{')) {
            const event = JSON.parse(rawValue)
            console.log('Parsed SSE event:', event) // Log parsed event

            if (event.type === 'client_id') {
              // Expect client_id now
              importProcessId.value = event.client_id // Store the active process ID
              localStorage.setItem('lastImportProcessId', event.client_id) // Save for potential resume/cancel later
              storedClientId.value = event.client_id
              logs.value.push({
                type: 'info',
                details: t('bulkImport.status.importStarted'),
                current: 0,
                word: '',
              })
            } else if (event.type === 'progress') {
              logs.value.push({
                type: event.success ? 'success' : 'error',
                details: event.success
                  ? t('bulkImport.status.importedSuccess')
                  : `${t('bulkImport.status.importError')} ${event.error}`,
                current: event.current,
                word: event.word,
              })

              if (logs.value.length > MAX_LOG_LINES) {
                logs.value.shift()
              }
            } else if (event.type === 'start') {
              logs.value.push({
                type: 'info',
                details: `Starting import of ${event.total} records.`, // Keep dynamic part
                current: 0,
                word: '',
              })
            } else if (event.type === 'progress') {
              logs.value.push({
                type: event.success ? 'success' : 'error',
                details: event.success
                  ? `Imported successfully. (${event.success_count}✓ ${event.error_count}✗)`
                  : `${t('bulkImport.status.importError')} ${event.error} (${event.success_count}✓ ${event.error_count}✗)`, // Keep dynamic part
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
              break // Exit loop on complete
            } else if (event.type === 'error') {
              // Fatal error from backend process
              setStatus(`${t('bulkImport.status.importFailed')}: ${event.error}`, 'error')
              logs.value.push({
                type: 'error',
                details: `${t('bulkImport.status.fatalError')} ${event.error}`,
              })
              break // Stop processing on fatal error
            } else {
              console.warn('Received unknown SSE event type:', event.type, event)
            }
            console.log(logs.value)
          } else if (rawValue.trim()) {
            // Log non-JSON, non-empty messages if any
            console.log('Received non-JSON SSE message:', rawValue)
          }
        } catch (error) {
          logs.value.push({
            type: 'error',
            message: 'Error processing SSE event',
            details: `Raw data: ${rawValue}. Error: ${error instanceof Error ? error.message : 'Unknown error'}`,
          })
          console.error('Error parsing SSE event:', error, 'Raw value:', rawValue)
          // Decide if we should break the loop on parsing error
          // break;
        }
      }
    }

    console.log('SSE reading loop finished. Aborted:', abortController.value?.signal.aborted)
    // Check if status was set, if not, maybe indicate an unexpected end
    if (!statusMessage.value && !abortController.value?.signal.aborted) {
      setStatus(t('bulkImport.status.unexpectedEnd'), 'error')
      logs.value.push({ type: 'error', details: t('bulkImport.status.connectionClosed') })
    }
  } catch (error) {
    // Handle fetch errors (network, CORS, etc.) or errors thrown explicitly
    console.error('Error during bulk import fetch/processing:', error)
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
    importProcessId.value = '' // Clear active process ID on completion/error
    localStorage.removeItem('lastImportProcessId') // Clear from storage
  }
}

// Set status message
const setStatus = (message, type = 'success') => {
  statusMessage.value = message
  statusType.value = type
}

// Computed properties
const canSubmit = computed(() => {
  return selectedLanguage.value && csvFile.value && !isLoading.value
})

// Load languages when component mounts
onMounted(() => {
  loadLanguages()
})

const cancelJob = async () => {
  if (!importProcessId.value) return // Use the active process ID

  isCancelling.value = true
  try {
    // Call the updated API function with the client ID
    const response = await cancelBulkImport(importProcessId.value)

    // Assuming cancelBulkImport now returns the raw fetch response or throws on error
    // Check if the status code indicates success (typically 2xx)
    if (!(response.status >= 200 && response.status < 300)) {
      // Use status and statusText for error message from Axios response
      throw new Error(`Failed to cancel job: ${response.status} ${response.statusText}`)
    }

    logs.value.push({
      type: 'info',
      details: t('bulkImport.status.cancellationRequested'),
      current: 0,
      word: '',
    })
    statusMessage.value = t('bulkImport.status.cancellationRequested') // Use same key for status
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

