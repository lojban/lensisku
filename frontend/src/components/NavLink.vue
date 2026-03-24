<template>
  <component
    :is="isActive ? 'span' : 'RouterLink'"
    :to="to"
    :class="{ 'nav-link': true, 'nav-link-active': isActive }"
  >
    <slot />
  </component>
</template>

<script setup>
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { localeCaptureGroupRegex } from '../config/locales'

const props = defineProps({
  to: {
    type: [String, Object],
    required: true,
  },
})

const route = useRoute()
const router = useRouter()

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

  const resolvedTargetLocation = router.resolve(props.to)

  const targetPathNoQuery = stripQueryParameters(resolvedTargetLocation.path)
  const targetNormalizedPath = stripLocalePrefix(targetPathNoQuery)

  return currentNormalizedPath === targetNormalizedPath
})
</script>

<style scoped>
.nav-link {
  text-decoration: none;
  color: #007bff;
}

.nav-link-active {
  color: #333;
  cursor: default;
}
</style>
