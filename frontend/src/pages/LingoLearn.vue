<template>
  <LingoLayout>
    <div class="flex justify-center py-12">
      <div class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent" />
    </div>
  </LingoLayout>
</template>

<script setup>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
useSeoHead({ title: t('lingo.learn') })

const LINGO_ACTIVE_COURSE_KEY = 'lingo_active_collection_id'

const router = useRouter()

onMounted(() => {
  try {
    const id = sessionStorage.getItem(LINGO_ACTIVE_COURSE_KEY)
    if (id) {
      router.replace(`/collections/${id}/lingo/levels`)
      return
    }
  } catch {}
  router.replace('/lingo/courses')
})
</script>
