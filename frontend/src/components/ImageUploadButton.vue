<template>

  <div class="image-upload flex flex-col items-center gap-4">
     <!-- Current Image Preview -->
    <div class="relative w-32 h-32">
       <img
        v-if="displayImage"
        :src="displayImage"
        :alt="`${username}'s profile picture`"
        class="w-32 h-32 rounded-full object-cover border-4 border-white shadow-lg"
      />
      <div
        v-else
        class="w-32 h-32 rounded-full bg-gray-200 flex items-center justify-center text-gray-400 border-4 border-white shadow-lg"
      >
         <User class="h-16 w-16" />
      </div>
       <!-- Upload Progress Overlay -->
      <div
        v-if="isUploading"
        class="absolute inset-0 rounded-full bg-black bg-opacity-50 flex items-center justify-center"
      >

        <div class="text-center">

          <div class="relative w-16 h-16 mx-auto">
             <!-- Circular Progress --> <svg
              class="w-full h-full transform -rotate-90"
              viewBox="0 0 100 100"
            >

              <circle cx="50" cy="50" r="45" fill="none" stroke="#4B5563" stroke-width="8" />

              <circle
                cx="50"
                cy="50"
                r="45"
                fill="none"
                stroke="#3B82F6"
                stroke-width="8"
                :stroke-dasharray="circumference"
                :stroke-dashoffset="circumference - (uploadProgress / 100) * circumference"
                class="transition-all duration-300"
              />
               </svg
            > <!-- Percentage Text -->
            <div class="absolute inset-0 flex items-center justify-center">
               <span class="text-white text-sm font-medium">{{ Math.round(uploadProgress) }}%</span>

            </div>

          </div>

        </div>

      </div>

    </div>
     <!-- Upload/Remove Controls -->
    <div class="flex gap-2">
       <label v-if="!isUploading" class="ui-btn--update cursor-pointer inline-flex items-center gap-2"
        > <input type="file" class="hidden" accept="image/*" @change="handleFileChange" /> <Upload
          class="h-4 w-4 shrink-0"
        /> {{ hasImage ? t('filters.changePhoto') : t('filters.uploadPhoto') }} </label
      > <button v-if="hasImage && !isUploading" class="ui-btn--delete inline-flex items-center gap-2" @click="handleRemove">
         <Trash2 class="h-4 w-4 shrink-0" /> {{ t('filters.removePhoto') }} </button
      >
    </div>

  </div>

</template>

<script setup lang="ts">
import { User, Upload, Trash2 } from 'lucide-vue-next'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useError } from '@/composables/useError'

const { t } = useI18n()

const { showError, clearError } = useError()
const props = defineProps({
  currentImage: {
    type: String,
    default: null,
  },
  username: {
    type: String,
    required: true,
  },
})

const emit = defineEmits(['upload', 'remove'])

const displayImage = ref(props.currentImage)
const uploadProgress = ref(0)
const isUploading = ref(false)
const circumference = computed(() => 2 * Math.PI * 45) // r = 45

const hasImage = computed(() => {
  return displayImage.value
})

watch(
  () => props.currentImage,
  (newValue) => {
    displayImage.value = newValue
  }
)

const handleFileChange = async (event) => {
  const file = event.target.files[0]
  if (!file) return

  // Reset states
  clearError()
  uploadProgress.value = 0

  // Validate file
  if (!file.type.startsWith('image/')) {
    showError(t('components.imageUploadButton.invalidType'))
    return
  }

  if (file.size > 5 * 1024 * 1024) {
    // 5MB limit
    showError(t('components.imageUploadButton.tooLarge'))
    return
  }

  try {
    isUploading.value = true

    // Read file as base64
    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        // Get base64 data without the data URL prefix
        const raw = e.target?.result
        const dataUrl = typeof raw === 'string' ? raw : ''
        const base64Data = dataUrl.split(',')[1]

        const imageData = {
          data: base64Data,
          mime_type: file.type,
        }

        // Create preview
        displayImage.value = URL.createObjectURL(file)

        // Simulate upload progress
        let progress = 0
        const progressInterval = setInterval(() => {
          progress += 5
          if (progress > 90) clearInterval(progressInterval)
          uploadProgress.value = Math.min(progress, 90)
        }, 100)

        // Emit the upload event with the properly formatted data
        emit('upload', imageData)

        // Complete the progress bar
        uploadProgress.value = 100
        setTimeout(() => {
          isUploading.value = false
          uploadProgress.value = 0
        }, 500)
      } catch (err) {
        showError(t('components.imageUploadButton.processError'))
        isUploading.value = false
        uploadProgress.value = 0
      }
    }

    reader.onerror = () => {
      showError(t('components.imageUploadButton.readError'))
      isUploading.value = false
      uploadProgress.value = 0
    }

    reader.readAsDataURL(file)
  } catch (err) {
    showError(t('components.imageUploadButton.uploadError'))
    isUploading.value = false
    uploadProgress.value = 0
  }
}

const handleRemove = () => {
  if (confirm(t('profile.removeConfirm'))) {
    displayImage.value = null
    emit('remove')
  }
}
</script>

<style scoped>
.image-upload input[type='file'] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  border: 0;
}
</style>

