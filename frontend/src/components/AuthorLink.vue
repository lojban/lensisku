<template>
  <a
    v-if="href"
    :href="href"
    target="_blank"
    rel="noopener noreferrer"
    class="text-blue-600 hover:underline"
    @click.stop
  >
    {{ username }}
  </a>
  <RouterLink
    v-else-if="linkLocal && username"
    :to="`/user/${encodeURIComponent(username)}`"
    class="text-blue-600 hover:underline"
    @click.stop
  >
    {{ username }}
  </RouterLink>
  <span v-else>{{ username }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

const MW_SUFFIX = '@mw.lojban.org'

const props = withDefaults(
  defineProps<{
    username?: string | null
    authorUrl?: string | null
    /** When there is no MediaWiki URL, link to a lensisku profile. */
    linkLocal?: boolean
  }>(),
  {
    username: '',
    authorUrl: null,
    linkLocal: true,
  }
)

const href = computed(() => {
  if (props.authorUrl) return props.authorUrl
  const name = props.username || ''
  if (!name.endsWith(MW_SUFFIX)) return ''
  const account = name.slice(0, -MW_SUFFIX.length)
  if (!account) return ''
  return `https://mw.lojban.org/papri/User:${encodeURIComponent(account.replace(/ /g, '_'))}`
})
</script>
