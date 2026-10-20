<template>
  <LingoLayout>
    <LingoFeedLayout>
      <div class="flex w-full flex-col items-center">
        <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
          <Trophy class="h-10 w-10 text-green-600" />
        </div>

        <h1 class="my-6 text-center text-2xl font-bold text-neutral-800">
          {{ t('lingo.leaderboard') }}
        </h1>
        <p class="mb-6 text-center text-lg text-slate-500">
          {{ t('lingo.leaderboardDescription') }}
        </p>

        <div v-if="isLoading" class="flex justify-center py-10">
          <div
            class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
          />
        </div>

        <p v-else-if="error" class="lingo-panel max-w-lg text-rose-600" role="alert">
          {{ error }}
        </p>

        <template v-else>
          <div class="mb-4 h-0.5 w-full max-w-lg rounded-full bg-slate-200" />

          <p v-if="entries.length === 0" class="lingo-panel max-w-lg">
            {{ t('lingo.leaderboardEmpty') }}
          </p>

          <div v-else class="w-full max-w-lg">
            <div
              v-for="entry in entries"
              :key="entry.user_id"
              class="flex w-full items-center rounded-xl p-2 px-4 hover:bg-gray-200/50"
              :class="{ 'bg-green-50': entry.is_you }"
            >
              <p class="mr-4 w-6 text-center font-bold text-lime-700">{{ entry.rank }}</p>
              <LingoAvatar
                :username="entry.username"
                :has-profile-image="entry.has_profile_image"
                :profile-image-url="entry.profile_image_url"
                :alt="entry.username"
              />
              <p class="min-w-0 flex-1 truncate font-bold text-neutral-800">
                <template v-if="entry.is_you">{{ t('lingo.you') }} ({{ entry.username }})</template>
                <template v-else>{{ entry.username }}</template>
              </p>
              <p class="shrink-0 text-slate-500">{{ entry.points }} XP</p>
            </div>
          </div>

          <div
            v-if="meOutsideTop"
            class="mt-6 w-full max-w-lg rounded-xl border-2 border-dashed border-green-300 bg-green-50/50 p-3"
          >
            <p class="mb-2 text-center text-xs font-bold uppercase tracking-wide text-green-700">
              {{ t('lingo.yourStanding') }}
            </p>
            <div class="flex w-full items-center px-1">
              <p class="mr-4 w-6 text-center font-bold text-lime-700">
                {{ meOutsideTop.rank || '—' }}
              </p>
              <LingoAvatar
                :username="meOutsideTop.username"
                :has-profile-image="meOutsideTop.has_profile_image"
                :profile-image-url="meOutsideTop.profile_image_url"
                :alt="meOutsideTop.username"
              />
              <p class="min-w-0 flex-1 truncate font-bold text-neutral-800">
                {{ t('lingo.you') }} ({{ meOutsideTop.username }})
              </p>
              <p class="shrink-0 text-slate-500">{{ meOutsideTop.points }} XP</p>
            </div>
          </div>
        </template>
      </div>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Trophy } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import LingoAvatar from '@/components/LingoAvatar.vue'
import { getLingoLeaderboard } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'

type LeaderboardEntry = {
  user_id: number
  username: string
  points: number
  rank: number
  has_profile_image: boolean
  profile_image_url?: string | null
  is_you: boolean
}

const { t } = useI18n()

const isLoading = ref(true)
const error = ref('')
const entries = ref<LeaderboardEntry[]>([])
const me = ref<LeaderboardEntry | null>(null)

const meOutsideTop = computed(() => {
  if (!me.value) return null
  if (entries.value.some((e) => e.user_id === me.value!.user_id)) return null
  return me.value
})

async function load() {
  isLoading.value = true
  error.value = ''
  try {
    const response = await getLingoLeaderboard({ limit: 10 })
    entries.value = response.data.entries || []
    me.value = response.data.me || null
  } catch (e) {
    console.error(e)
    error.value = t('lingo.leaderboardLoadError')
    entries.value = []
    me.value = null
  } finally {
    isLoading.value = false
  }
}

useSeoHead({ title: t('lingo.leaderboard') })
onMounted(load)
</script>
