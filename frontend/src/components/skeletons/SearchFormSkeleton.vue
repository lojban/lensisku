<template>
  <!-- Mirror SearchForm.vue geometry so the Discussions trailing control matches width/height. -->
  <div class="search-form max-w-3xl mx-auto w-full" aria-hidden="true">
    <div class="relative flex w-full min-w-0">
      <div class="flex-1 min-w-0">
        <div
          class="input-field h-10 w-full min-w-0 sm:min-w-[200px] rounded-r-none bg-gray-200 border-gray-200 shadow-none pointer-events-none"
        />
      </div>
      <div
        class="dropdown-trigger dropdown-trigger--search-bar-trailing w-auto max-w-none rounded-l-none rounded-r-full shrink-0 pointer-events-none bg-gray-200 border-gray-200"
      >
        <!-- Invisible real label keeps locale-accurate button width -->
        <span class="invisible select-none whitespace-nowrap">{{
          t('flashcardStudy.wavesButton')
        }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>
