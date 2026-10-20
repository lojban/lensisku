<template>
  <LingoLayout>
    <div class="flex justify-center py-12">
      <div
        class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
      />
    </div>
  </LingoLayout>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { t } = useI18n()
useSeoHead({ title: t('lingo.learn') })

const router = useRouter()
const { activeCourseId } = useLingoProgress()

onMounted(() => {
  const id = activeCourseId.value
  if (id) {
    router.replace(`/collections/${id}/lingo/levels`)
    return
  }
  router.replace('/lingo/courses')
})
</script>
