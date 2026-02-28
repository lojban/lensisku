<template>
  <div ref="contentRef" :key="contentKey" class="mathjax-content break-words" />
</template>

<script setup>
import { marked } from 'marked'
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
  return text
    .replace(/(https?:)(\\_){2}/g, '$1//')
    .replace(/(https?)__(?=[^\s\]])/g, '$1://')
}

const renderContent = async () => {
  if (!contentRef.value || !props.content) return

  let finalContent = props.content

  if (props.enableMarkdown) {
    finalContent = normalizeLinkAnchors(finalContent)
    // First split content into LaTeX and non-LaTeX parts
    const parts = props.content.split(/(\$[^$]+\$)/)
    finalContent = parts
      .map((part) => {
        // Skip markdown processing for LaTeX parts
        if (part.startsWith('$') && part.endsWith('$')) {
          return part
        }

        // Process non-LaTeX parts with markdown
        const extensions = [
          {
            name: 'curlyLink',
            level: 'inline',
            start(src) {
              return src.indexOf('{')
            },
            tokenizer(src) {
              const rule = /^{([^}]+)}/
              const match = rule.exec(src)
              if (match) {
                return {
                  type: 'curlyLink',
                  raw: match[0],
                  text: match[1].trim(),
                  href: match[1].trim(),
                }
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

        const renderer = {
            link(href, title, text) {
              // Check if this is a valsi link
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
            // Default link handling
            return `<a href="${href.href}" title="${href.title || ''}" class="text-blue-600 hover:text-blue-800 hover:underline">${href.text}</a>`
          },
        }

        marked.use({ extensions, renderer })
        return marked(part)
      })
      .join('')
  }

  // Apply highlighting if searchTerm is provided
  if (props.searchTerm && finalContent) {
    const escapedSearchTerm = props.searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedSearchTerm})`, 'gi');
    finalContent = finalContent.replace(regex, '<mark>$1</mark>');
  }

  // Handle newlines if markdown is not enabled (AFTER highlighting)
  if (!props.enableMarkdown && finalContent) {
    finalContent = finalContent.replace(/\n/g, '<br>');
  }

  contentRef.value.innerHTML = finalContent
  if (
    finalContent.includes('$') ||
    finalContent.includes('\\[') ||
    finalContent.includes('\\(')
  ) {
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

:deep(.curly-quotes::before) {
  content: "«";
  display: inline-block;
  margin-right: 0.2em;
}

:deep(.curly-quotes::after) {
  content: "»";
  display: inline-block;
  margin-left: 0.2em;
}
</style>
