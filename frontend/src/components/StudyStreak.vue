<template>
  <div
    v-if="auth.state.isLoggedIn"
    class="card-base card-compact card-streak min-h-[11rem] p-4 sm:min-h-[13rem] sm:p-5"
  >
    <template v-if="!isLoadingStreak && streakData">
      <div class="card-streak-header">
        <h3 class="card-streak-title select-none">{{ t('collectionList.studyStreak') }}</h3>

        <div class="card-streak-meta">
          <span class="font-semibold text-gray-700">{{
            t('collectionList.currentStreakWithDays', {
              count: streakData.current_streak,
            })
          }}</span>
        </div>
      </div>

      <div class="card-streak-week-grid">
        <div
          v-for="day in streakData.daily_progress.slice(0, 7).reverse()"
          :key="day.date"
          class="card-streak-day"
        >
          <div class="card-streak-day-label">{{ streakWeekdayShort(day.date) }}</div>

          <div
            class="card-streak-day-count"
            :class="
              day.reviews_count > 0 ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-400'
            "
          >
            {{ day.reviews_count }}
          </div>

          <div
            class="card-streak-day-points"
            :title="t('collectionList.points', { count: day.points })"
            :aria-label="t('collectionList.points', { count: day.points })"
          >
            {{ streakPointsEmoji(day.points) }}
          </div>
        </div>
      </div>
    </template>

    <div v-else class="streak-skeleton animate-pulse" aria-hidden="true">
      <div class="card-streak-header">
        <div class="h-6 w-1/3 min-w-[6rem] max-w-[12rem] rounded bg-gray-200" />

        <div class="h-4 w-28 rounded bg-gray-100 sm:w-36" />
      </div>

      <div class="card-streak-week-grid">
        <div v-for="i in 7" :key="i" class="card-streak-day">
          <div class="card-streak-skeleton-line" />

          <div class="card-streak-day-count bg-gray-100" />

          <div class="card-streak-skeleton-line card-streak-skeleton-line--points" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getStreak } from '@/api'
import { useAuth } from '@/composables/useAuth'

interface StreakDay {
  date: string
  reviews_count: number
  points: number
}

interface StreakData {
  current_streak: number
  daily_progress: StreakDay[]
}

const auth = useAuth()
const { t, locale, tm } = useI18n()

const streakData = ref<StreakData | null>(null)
const isLoadingStreak = ref(false)

/** Maps daily points to a threshold-based emoji indicator. */
const streakPointsEmoji = (points: number): string => {
  if (points <= 0) return '😴'
  if (points < 10) return '🟦'
  if (points < 25) return '⭐'
  if (points < 50) return '🔥'
  return '🏆'
}

/** Gregorian weekdays (JS getDay 0=Sun..6=Sat): color lujvo from sampu vlaste (xunre…zirpu + dei). */
const streakWeekdayShort = (isoDate: string) => {
  const d = new Date(isoDate)
  if (locale.value !== 'jbo') {
    return d.toLocaleDateString(locale.value, { weekday: 'short' })
  }
  const labels = tm('collectionList.weekdayGregorian') as Record<string, string>
  return labels[String(d.getDay())] ?? d.toLocaleDateString('en-US', { weekday: 'short' })
}

const fetchStreakData = async () => {
  if (!auth.state.isLoggedIn) return

  isLoadingStreak.value = true
  try {
    const response = await getStreak(7) // Get last 7 days
    streakData.value = response.data as StreakData
  } catch (error) {
    console.error('Error fetching streak data:', error)
  } finally {
    isLoadingStreak.value = false
  }
}

onMounted(fetchStreakData)
</script>
