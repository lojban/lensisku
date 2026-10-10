<template>
  <div class="space-y-3">
    <div v-if="loading && !rows.length" class="space-y-3" role="status">
      <div v-for="n in 3" :key="n" class="h-36 rounded-xl bg-gray-100 animate-pulse" />
    </div>
    <ActivityChanges
      v-else-if="!error || rows.length"
      :grouped-changes="groupedChanges"
      :format-date="formatDate"
    />

    <div
      v-if="error"
      class="rounded-lg border border-red-100 bg-red-50 p-4 text-sm text-red-800"
      role="alert"
    >
      <p>{{ t('activityFeed.error') }}</p>
      <button type="button" class="mt-2 underline" @click="load()">
        {{ t('activityFeed.retry') }}
      </button>
    </div>
    <div v-else-if="rows.length" class="text-center py-3">
      <Button v-if="cursor" variant="neutral" :disabled="loading" @click="load(true)">
        {{ loading ? t('activityFeed.loading') : t('activityFeed.loadMore') }}
      </Button>
      <p v-else-if="!loading" class="text-sm text-gray-500">{{ t('activityFeed.end') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Button } from '@packages/ui'
import { getRecentChanges } from '@/api'
import ActivityChanges from '@/components/activity/ActivityChanges.vue'
import { useDateFormat } from '@/composables/useDateFormat'
import { useNewsUnread } from '@/composables/useNewsUnread'
import { useI18n } from 'vue-i18n'

type ChangeRow = { time: number; [key: string]: unknown }
const { t } = useI18n()
const { formatDate } = useDateFormat()
const { markNewsOpened } = useNewsUnread()
const rows = ref<ChangeRow[]>([])
const cursor = ref<string | null>(null)
const loading = ref(false)
const error = ref(false)
let controller: AbortController | undefined

const groupedChanges = computed(() => {
  const groups = new Map<string, { date: Date; changes: ChangeRow[] }>()
  for (const row of rows.value) {
    const date = new Date(row.time * 1000)
    const key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
    const group = groups.get(key) ?? { date, changes: [] }
    group.changes.push(row)
    groups.set(key, group)
  }
  return [...groups.values()].sort((a, b) => b.date.getTime() - a.date.getTime())
})

async function load(append = false) {
  if (loading.value || (append && !cursor.value)) return
  controller?.abort()
  controller = new AbortController()
  const request = controller
  loading.value = true
  error.value = false
  try {
    const response = await getRecentChanges(
      { limit: 20, types: 'news', ...(append && cursor.value ? { after: cursor.value } : {}) },
      request.signal
    )
    if (request !== controller) return
    const incoming = (response.data.changes ?? []) as ChangeRow[]
    const seen = new Set(
      append ? rows.value.map((row) => `${row.time}:${row.change_type}:${row.word}`) : []
    )
    rows.value = [
      ...(append ? rows.value : []),
      ...incoming.filter((row) => {
        const key = `${row.time}:${row.change_type}:${row.word}`
        if (seen.has(key)) return false
        seen.add(key)
        return true
      }),
    ]
    cursor.value = response.data.next_cursor ?? null
    markNewsOpened()
  } catch {
    if (request === controller && !request.signal.aborted) error.value = true
  } finally {
    if (request === controller) loading.value = false
  }
}

onMounted(() => void load())
onBeforeUnmount(() => controller?.abort())
</script>
