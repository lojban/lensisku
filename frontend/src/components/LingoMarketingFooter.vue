<template>
  <footer class="lingo-marketing-footer hidden border-t-2 border-slate-200 bg-white py-4 lg:block">
    <div class="mx-auto flex max-w-screen-lg items-center justify-center gap-6 text-sm text-slate-500">
      <span>{{ t('lingo.learnLanguages', 'Learn languages with Lingo') }}</span>
    </div>
  </footer>
</template>

<script setup>
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>
