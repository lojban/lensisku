<template>
  <component
    :is="isActive ? 'span' : 'RouterLink'"
    :to="localizedTo"
    :class="['nav-link', isActive ? 'nav-link-active' : 'text-nav-link']"
  >
    <slot />
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useLocalePath } from '@/composables/useLocalePath'
import { localeCaptureGroupRegex } from '../config/locales'

const props = defineProps({
  to: {
    type: [String, Object],
    required: true,
  },
})

const route = useRoute()
const router = useRouter()

const localePath = useLocalePath()
const localizedTo = computed(() => {
  const to = props.to
  if (typeof to === 'string')
    return to.startsWith('/') && !to.match(/^\/(en|jbo|ru|ja|zh)(?=\/|$)/) ? localePath(to) : to
  if ('path' in to && typeof to.path === 'string' && !to.path.match(/^\/(en|jbo|ru|ja|zh)(?=\/|$)/))
    return { ...to, path: localePath(to.path) }
  return to
})

const stripQueryParameters = (path) => path.split('?')[0]

const stripLocalePrefix = (path) => {
  const match = path.match(localeCaptureGroupRegex)
  if (match && match[0]) {
    let strippedPath = path.substring(match[0].length)
    if (!strippedPath) {
      return '/'
    }
    if (!strippedPath.startsWith('/')) {
      strippedPath = '/' + strippedPath
    }
    return strippedPath
  }
  return path
}

const isActive = computed(() => {
  const currentPathNoQuery = stripQueryParameters(route.path)
  const currentNormalizedPath = stripLocalePrefix(currentPathNoQuery)

  const resolvedTargetLocation = router.resolve(localizedTo.value)

  const targetPathNoQuery = stripQueryParameters(resolvedTargetLocation.path)
  const targetNormalizedPath = stripLocalePrefix(targetPathNoQuery)

  return currentNormalizedPath === targetNormalizedPath
})
</script>

<style scoped>
.nav-link {
  text-decoration: none;
}

.nav-link-active {
  color: #333;
  cursor: default;
}
</style>
