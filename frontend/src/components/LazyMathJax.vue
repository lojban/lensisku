<template>

  <div ref="contentRef" :key="contentKey" class="mathjax-content break-words" />

</template>

<script setup lang="ts">
import { Marked } from 'marked'
import type { RendererObject } from 'marked'
import { ref, onMounted, watch, nextTick, onBeforeUnmount } from 'vue'

const props = defineProps({
  content: {
    type: String,
    required: true,
  },
  enableMarkdown: {
    type: Boolean,
    default: false,
  },
  langId: {
    type: String,
    default: '',
  },
  username: {
    type: String,
    default: '',
  },
  searchTerm: {
    type: String,
    default: '',
  },
  curlyLinkClass: {
    type: String,
    default: 'text-blue-600 hover:text-blue-800', // Default link styling
  },
  /** `{valsi}` → /valsi/ links. Disable in assistant chat so `$x_{1}$` is not parsed as `{1}`. */
  enableCurlyLinks: {
    type: Boolean,
    default: true,
  },
})

const contentRef = ref(null)
const contentKey = ref(0)
let observer = null
const renderQueue = new Set()
let isProcessingQueue = false
let renderQueued = false

const processRenderQueue = async () => {
  if (isProcessingQueue || renderQueue.size === 0) return

  isProcessingQueue = true
  try {
    const BATCH_SIZE = 3
    const elements = Array.from(renderQueue)

    for (let i = 0; i < elements.length; i += BATCH_SIZE) {
      const batch = elements.slice(i, i + BATCH_SIZE)
      if (window.MathJax?.typesetPromise) {
        await window.MathJax.typesetPromise(batch)
        batch.forEach((el) => renderQueue.delete(el))
      }
      if (i + BATCH_SIZE < elements.length) {
        await new Promise((resolve) => setTimeout(resolve, 50))
      }
    }
  } finally {
    isProcessingQueue = false
    if (renderQueue.size > 0) {
      processRenderQueue()
    }
  }
}

const queueForRendering = (element) => {
  if (!element || renderQueue.has(element)) return
  renderQueue.add(element)
  if (!isProcessingQueue) {
    processRenderQueue()
  }
}

/** Normalize link anchor text: fix "https\_\_" or "https__" in markdown (serializer escape bug) */
function normalizeLinkAnchors(text) {
  if (!text || typeof text !== 'string') return text
  return text.replace(/(https?:)(\\_){2}/g, '$1//').replace(/(https?)__(?=[^\s\]])/g, '$1://')
}

const renderContent = async () => {
  if (!contentRef.value || !props.content) return

  let finalContent = props.content

  if (props.enableMarkdown) {
    finalContent = normalizeLinkAnchors(finalContent)

    // Preserve multiple empty lines by converting them to <br> tags.
    // marked will collapse \n\n into paragraph separations and ignore extra \n's.
    finalContent = finalContent.replace(/\n{3,}/g, (match) => {
      return '\n\n' + '<br>'.repeat(match.length - 2) + '\n\n'
    })

    const extensions = props.enableCurlyLinks
      ? [
          {
            name: 'curlyLink',
            level: 'inline',
            start(src) {
              return src.indexOf('{')
            },
            tokenizer(src) {
              const rule = /^{([^}]+)}/
              const match = rule.exec(src)
              if (!match) return
              const inner = match[1].trim()
              // LaTeX subscripts like $x_{1}$ — do not treat as /valsi/ link
              if (/^\d+$/.test(inner)) return
              return {
                type: 'curlyLink',
                raw: match[0],
                text: inner,
                href: inner,
              }
            },
            renderer(token) {
              const url = new URL(`/valsi/${token.href.replace(/ /g, '_')}`, window.location.origin)
              if (props.langId) url.searchParams.set('langid', props.langId)
              if (props.username) url.searchParams.set('username', props.username)
              return `<a
                    href="${url.toString()}" class="${props.curlyLinkClass} hover:underline inline curly-quotes">${token.text}</a>`
            },
          },
        ]
      : []

    const renderer = {
      link(href, title, text) {
        if (href?.href?.startsWith('/valsi/')) {
          const word = href.href.split('/valsi/')[1]
          const url = new URL(`/valsi/${word.replace(/ /g, '_')}`, window.location.origin)
          if (props.langId) url.searchParams.set('langid', props.langId)
          if (props.username) url.searchParams.set('username', props.username)
          return `<a
                        href="${url.toString()}"
                        class="text-blue-600 hover:text-blue-800 hover:underline inline">
                        ${text ?? href.text}
                    </a>`
        }
        return `<a href="${href.href}" title="${href.title || ''}" class="text-blue-600 hover:text-blue-800 hover:underline">${href.text}</a>`
      },
    }

    const mdParser = new Marked()
    mdParser.use({ extensions, renderer: renderer as RendererObject })
    const parsed = mdParser.parse(finalContent)
    finalContent = typeof parsed === 'string' ? parsed : await parsed
  }

  // Apply highlighting if searchTerm is provided
  if (props.searchTerm && finalContent) {
    const escapedSearchTerm = props.searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(`(${escapedSearchTerm})`, 'gi')
    finalContent = finalContent.replace(regex, '<mark>$1</mark>')
  }

  // Handle newlines if markdown is not enabled (AFTER highlighting)
  if (!props.enableMarkdown && finalContent) {
    finalContent = finalContent.replace(/\n/g, '<br>')
  }

  contentRef.value.innerHTML = finalContent

  if (props.enableMarkdown) {
    contentRef.value.querySelectorAll('table').forEach((table) => {
      if (table.parentElement?.classList?.contains('mathjax-table-wrap')) return
      const wrap = document.createElement('div')
      wrap.className = 'mathjax-table-wrap'
      table.parentNode.insertBefore(wrap, table)
      wrap.appendChild(table)
    })
  }

  if (finalContent.includes('$') || finalContent.includes('\\[') || finalContent.includes('\\(')) {
    queueForRendering(contentRef.value)
  }
}

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting && !renderQueued) {
          renderQueued = true
          renderContent()
        }
      })
    },
    { rootMargin: '50px', threshold: 0 }
  )

  if (contentRef.value) {
    observer.observe(contentRef.value)
  }
})

watch(
  () => props.content,
  () => {
    contentKey.value++
    renderQueued = false
    nextTick(() => {
      if (contentRef.value && observer) {
        observer.observe(contentRef.value)
      }
    })
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  if (observer) {
    observer.disconnect()
  }
  if (contentRef.value) {
    renderQueue.delete(contentRef.value)
  }
})
</script>

<style scoped>
.mathjax-content {
  min-height: 1em;
  overflow-wrap: anywhere;
}

:deep(.MathJax) {
  margin: 0.25em 0;
}

.mathjax-content {
  @apply inline;
}

/* Force long text (e.g. valsi) inside links to break by character.
   inline-block + max-width so the link has a width; then word-break wraps the text. */
.mathjax-content :deep(a) {
  display: inline-block !important;
  max-width: 100% !important;
  overflow-wrap: anywhere !important;
  word-break: break-word !important;
}

:deep(p) {
  @apply inline;
}

/* Two consecutive <p> blocks: force block display so they stack vertically */
:deep(p + p),
:deep(p:has(+ p)) {
  display: block !important;
}

.mathjax-content :deep(> *) {
  @apply inline;
}

/* Override for blockquotes */

:deep(blockquote:has(+ blockquote)) {
  @apply mt-2 mb-0;
}

/* Remove margins between consecutive blockquotes */
:deep(blockquote + blockquote) {
  @apply mt-0 mb-0;
}

:deep(blockquote:not(:has(+ blockquote))) {
  @apply mb-2;
}

:deep(blockquote) {
  @apply mt-2;
}

:deep(blockquote p) {
  @apply block;
}

/* Ensure blockquote overrides inline styles */
.mathjax-content :deep(> blockquote) {
  display: block !important;
}

.mathjax-content :deep(> blockquote > *) {
  display: block !important;
}

/* Pipe tables: scroll wide content; root `inline` rule would break layout without wrapper */
.mathjax-content :deep(.mathjax-table-wrap) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  margin: 0.5rem 0;
}

.mathjax-content :deep(.mathjax-table-wrap table) {
  display: table !important;
  width: max-content;
  min-width: min(100%, 28rem);
  max-width: none;
  border-collapse: collapse;
  font-size: 0.95em;
}

.mathjax-content :deep(> table) {
  display: table !important;
  width: 100%;
  max-width: 100%;
  border-collapse: collapse;
  margin: 0.5rem 0;
  font-size: 0.95em;
}

.mathjax-content :deep(table th),
.mathjax-content :deep(table td) {
  border: 1px solid rgb(229 231 235);
  padding: 0.25rem 0.5rem;
  vertical-align: top;
  text-align: left;
  min-width: 7.5rem;
}

:deep(.curly-quotes::before) {
  content: '«';
  display: inline-block;
  margin-right: 0.2em;
}

:deep(.curly-quotes::after) {
  content: '»';
  display: inline-block;
  margin-left: 0.2em;
}
</style>

