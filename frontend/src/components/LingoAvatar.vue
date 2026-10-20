<template>
  <div class="lingo-avatar">
    <img v-if="src && !failed" :src="src" :alt="alt || username" @error="failed = true" />
    <span v-else aria-hidden="true">{{ initials }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { getProfileImage } from '@/api'

const props = withDefaults(
  defineProps<{
    username: string
    /** Present when the leaderboard (or other) API reports an avatar. */
    hasProfileImage?: boolean
    /** Optional API-relative URL; presence alone also opts into loading the image. */
    profileImageUrl?: string | null
    alt?: string
  }>(),
  {
    hasProfileImage: false,
    profileImageUrl: null,
    alt: '',
  }
)

const failed = ref(false)

watch(
  () => [props.username, props.profileImageUrl, props.hasProfileImage] as const,
  () => {
    failed.value = false
  }
)

const src = computed(() => {
  if (props.hasProfileImage || props.profileImageUrl) {
    return getProfileImage(props.username, { cached: true })
  }
  return ''
})

const initials = computed(() => {
  const name = (props.username || '?').trim()
  return name.slice(0, 2).toUpperCase() || '?'
})
</script>
