<template>
  <div class="wiki-article-body" @click="onBodyClick">
    <LazyMathJax :content="renderedMarkdown" :enable-markdown="true" @rendered="emit('rendered')" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import LazyMathJax from '@/components/LazyMathJax.vue'

const props = defineProps<{ content: string }>()
const emit = defineEmits<{ rendered: [] }>()
const router = useRouter()

const withHeadingAnchors = (content: string) => {
  const ids = new Set<string>()
  return content.replace(
    /^(#{1,6})\s+(.+?)\s*#*\s*$/gm,
    (heading, _marks: string, label: string) => {
      const plain = label
        .replace(/<[^>]*>/g, '')
        .replace(/[`*_~]/g, '')
        .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
      const baseId = plain
        .trim()
        .toLowerCase()
        .replace(/[^\p{L}\p{N}\s-]/gu, '')
        .replace(/\s+/g, '-')
      const base = baseId || 'section'
      let id = base
      let duplicate = 1
      while (ids.has(id)) id = `${base}-${duplicate++}`
      ids.add(id)
      return `<span id="${id}"></span>\n${heading}`
    }
  )
}

const renderedMarkdown = computed(() =>
  withHeadingAnchors(props.content).replace(/\[\[([^\]]+)\]\](?!\()/g, (_match, page: string) => {
    const [rawTitle, ...anchorParts] = page.trim().split('#')
    const title = rawTitle.trim()
    const anchor = anchorParts.join('#').trim()
    const href = `/wiki/${encodeURIComponent(title.replace(/ /g, '_'))}${anchor ? `#${encodeURIComponent(anchor)}` : ''}`
    return `[${title}](${href})`
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

<style scoped>
.wiki-article-body :deep(.mathjax-content) {
  display: block !important;
  line-height: 1.7;
}

.wiki-article-body :deep(p) {
  display: block !important;
  margin: 0.65rem 0 !important;
}

.wiki-article-body :deep(h1),
.wiki-article-body :deep(h2),
.wiki-article-body :deep(h3),
.wiki-article-body :deep(h4),
.wiki-article-body :deep(h5),
.wiki-article-body :deep(h6) {
  display: block !important;
  margin: 1.5rem 0 0.65rem !important;
  font-weight: 650 !important;
  line-height: 1.35 !important;
}

.wiki-article-body :deep(ul),
.wiki-article-body :deep(ol) {
  display: block !important;
  margin: 0.75rem 0 0.75rem 1.5rem !important;
  padding-left: 1rem;
}

.wiki-article-body :deep(ul) {
  list-style: disc;
}
.wiki-article-body :deep(ol) {
  list-style: decimal;
}
.wiki-article-body :deep(li) {
  display: list-item !important;
  margin: 0.25rem 0;
}

.wiki-article-body :deep(blockquote) {
  display: block !important;
  margin: 1rem 0 !important;
  padding: 0.25rem 1rem;
  border-left: 3px solid rgb(203 213 225);
  color: rgb(71 85 105);
}

.wiki-article-body :deep([id]) {
  scroll-margin-top: 1rem;
}
</style>
