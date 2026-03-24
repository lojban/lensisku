<template>

  <div>

    <div class="flex items-center justify-between">
       <label for="definition" class="block text-sm font-medium text-blue-700"
        > {{ label || t('imageUpload.image') }} </label
      > <button
        v-if="modelValue || loadedImage"
        type="button"
        class="text-sm text-red-600 hover:text-red-700"
        @click="handleRemove"
      >
         {{ t('imageUpload.removeImage') }} </button
      > <span v-else-if="note" class="text-xs text-gray-500"> {{ note }} </span>
    </div>
     <!-- Image Preview -->
    <div v-if="modelValue || loadedImage" class="relative flex justify-center">
       <img
        :src="
          modelValue?.dataUri ||
          (modelValue ? `data:${modelValue.mime_type};base64,${modelValue.data}` : previewUrl)
        "
        alt="Preview"
        class="max-h-64 rounded-lg object-contain bg-gray-100"
      />
      <div
        class="absolute inset-0 bg-black bg-opacity-0 hover:bg-opacity-10 transition-opacity rounded-lg"
      />

    </div>
     <!-- Upload Button -->
    <div
      v-if="!modelValue && !loadedImage"
      ref="dropZoneRef"
      class="flex justify-center px-6 pt-5 pb-6 border-2 border-dashed rounded-lg transition-colors"
      :class="{
        'border-blue-400 bg-blue-50': isOverDropZone,
        'border-gray-300': !isOverDropZone,
      }"
    >

      <div class="space-y-1 text-center">
         <ImagePlus class="mx-auto h-12 w-12 text-gray-300" :stroke-width="1" />
        <div class="flex text-sm text-gray-600">
           <label
            class="relative cursor-pointer rounded-md font-medium text-blue-600 hover:text-blue-500"
            > <span>{{ t('imageUpload.uploadPrompt') }}</span
            > <input type="file" class="sr-only" accept="image/*" @change="handleFileSelect" />
            </label
          >
          <p class="pl-1"> {{ t('imageUpload.dragDrop') }} </p>

        </div>

        <p class="text-xs text-gray-500"> {{ t('imageUpload.fileTypes') }} </p>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { ImagePlus } from 'lucide-vue-next'
import { useDropZone } from '@vueuse/core'
import { ref, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

import { useError } from '../composables/useError'

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
  side: {
    type: String,
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
  definitionId: {
    type: String,
    default: null,
  },
  hasExistingImage: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:modelValue', 'image-loaded', 'remove-image'])

const previewUrl = ref('')
const loadedImage = ref(null)
const isLoading = ref(false)
const dropZoneRef = ref(null)

const { isOverDropZone } = useDropZone(dropZoneRef, (files) => {
  if (files && files.length > 0) {
    processFile(files[0])
  }
})

const clearImage = () => {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = ''
  }
  loadedImage.value = null
}

const handleRemove = () => {
  clearImage()
  emit('update:modelValue', null)
  emit('remove-image')
}

const loadExistingImage = async () => {
  if (!props.hasExistingImage || !(props.definitionId || props.collectionId) || isLoading.value)
    return

  try {
    isLoading.value = true
    clearImage() // Clear any existing image data
    clearError()

    const url = props.itemId
      ? `/api/collections/${props.collectionId}/items/${props.itemId}/image/${props.side}`
      : `/api/jbovlaste/definition/${props.definitionId}/image`

    const response = await fetch(url)
    if (!response.ok) {
      throw new Error('Failed to fetch image')
    }

    const blob = await response.blob()

    // Create object URL for preview
    previewUrl.value = URL.createObjectURL(blob)

    // Convert blob to base64
    const reader = new FileReader()
    reader.onload = (e) => {
      const raw = e.target?.result
      const dataUrl = typeof raw === 'string' ? raw : ''
      const base64String = dataUrl.split(',')[1]
      loadedImage.value = {
        data: base64String,
        mime_type: blob.type,
      }
      emit('image-loaded', loadedImage.value)
    }
    reader.readAsDataURL(blob)
  } catch (e) {
    showError('Failed to load existing image')
    console.error('Error loading image:', e)
    clearImage()
  } finally {
    isLoading.value = false
  }
}

const validateFile = (file) => {
  if (!['image/jpeg', 'image/png', 'image/gif', 'image/webp'].includes(file.type)) {
    throw new Error('Invalid file type. Please upload a PNG, JPG, or GIF image.')
  }

  if (file.size > 5 * 1024 * 1024) {
    throw new Error('File size exceeds 5MB limit.')
  }
}

const processFile = async (file) => {
  try {
    validateFile(file)
    clearError()
    clearImage() // Clear any existing image data

    // Create file preview
    previewUrl.value = URL.createObjectURL(file)

    // Convert to base64
    const reader = new FileReader()
    reader.onload = (e) => {
      const raw = e.target?.result
      const dataUrl = typeof raw === 'string' ? raw : ''
      const base64String = dataUrl.split(',')[1]
      const imageObj = {
        data: base64String,
        mime_type: file.type,
      }
      emit('update:modelValue', imageObj)
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

// Watch for changes in modelValue
watch(
  () => props.modelValue,
  (newValue) => {
    if (!newValue) {
      clearImage()
    }
  }
)

// Watch for changes in hasExistingImage
watch(
  () => props.hasExistingImage,
  (newValue) => {
    if (newValue && props.definitionId) {
      loadExistingImage()
    } else if (!newValue) {
      clearImage()
    }
  },
  { immediate: true }
)

// Watch for changes in definitionId
watch(
  () => props.definitionId,
  (newValue) => {
    if (newValue && props.hasExistingImage) {
      loadExistingImage()
    }
  }
)
watch(
  () => props.itemId,
  (newValue) => {
    if (newValue && props.side) {
      loadExistingImage()
    }
  }
)

onMounted(() => {
  if (
    props.hasExistingImage &&
    (props.definitionId || (props.collectionId && props.itemId && props.side))
  ) {
    loadExistingImage()
  }
})
</script>

