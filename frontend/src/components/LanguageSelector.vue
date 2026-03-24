<template>
  <div ref="dropdownRef" class="relative inline-flex items-center">
    <button
      class="text-sm text-white/70 hover:text-white transition-colors"
      @click.stop="showDropdown = !showDropdown"
    >
      {{ currentLanguage.name }}
    </button>

    <div
      v-if="showDropdown"
      class="absolute overflow-y-auto max-h-[205px] py-2 w-48 right-0 top-full mt-1 bg-gray-900 rounded-md shadow-lg z-50"
    >
      <button
        v-for="language in [...LANGUAGES].sort((a, b) => a.name.localeCompare(b.name))"
        :key="language.id"
        class="w-full items-center flex gap-3 px-3 py-1 hover:bg-gray-800"
        @click="setLanguage(language.id)"
      >
        <img class="w-5" :src="language.flag" :alt="language.name" />
        <span class="text-xs">{{ language.name }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { LANGUAGES } from '../composables/tiktoknu/languages'
import { useLocalization } from '../composables/tiktoknu/useLocalization'

const { currentLanguage, setLanguage } = useLocalization()
const showDropdown = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

const handleClickOutside = (event: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    showDropdown.value = false
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>
