<template>
  <div class="wiki-article-body" @click="onBodyClick">
    <LazyMathJax :content="renderedMarkdown" :enable-markdown="true" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import LazyMathJax from '@/components/LazyMathJax.vue'

const props = defineProps<{ content: string }>()
const router = useRouter()

const renderedMarkdown = computed(() =>
  props.content.replace(/\[\[([^\]]+)\]\](?!\()/g, (_match, page: string) => {
    const title = page.trim()
    return `[${title}](/wiki/${encodeURIComponent(title.replace(/ /g, '_'))})`
  })
)

function onBodyClick(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  const anchor = target?.closest('a') as HTMLAnchorElement | null
  if (!anchor) return
  // Links inside a preview should not trigger the result card's navigation.
  event.stopPropagation()
  const href = anchor.getAttribute('href')
  if (!href?.startsWith('/wiki/')) return
  event.preventDefault()
  router.push(href)
}
</script>
