<template>
  <span class="lingo-hint-phrase" :lang="lang || undefined" :dir="dir || undefined">
    <template v-for="(tok, i) in tokens" :key="i">
      <span v-if="tok.kind === 'space'">{{ tok.text }}</span>
      <button
        v-else
        type="button"
        class="lingo-hint-token"
        :aria-expanded="openIndex === i"
        :aria-label="glossFor(tok.text) ? undefined : tok.text"
        @click.stop="toggle(i)"
      >
        {{ tok.text }}
        <span
          v-if="openIndex === i && glossFor(tok.text)"
          class="lingo-hint-popover"
          role="tooltip"
          :lang="glossLang || undefined"
          :dir="glossDir || undefined"
        >
          {{ glossFor(tok.text) }}
          <span class="lingo-hint-popover__arrow" aria-hidden="true" />
        </span>
      </button>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps({
  text: { type: String, default: '' },
  /** Full opposite-side phrase shown when a token has no per-token gloss. */
  phraseGloss: { type: String, default: '' },
  /** Optional per-token glosses (case-insensitive / apostrophe-normalized keys). */
  tokenGlosses: { type: Object, default: () => ({}) },
  /** Auto-open first token once when true (caller gates on sessionStorage). */
  autoOpenFirst: { type: Boolean, default: false },
  lang: { type: String, default: '' },
  dir: { type: String, default: '' },
  glossLang: { type: String, default: '' },
  glossDir: { type: String, default: '' },
})

const emit = defineEmits(['auto-opened'])

const openIndex = ref<number | null>(null)

function normalizeTokenKey(s: string): string {
  return s
    .trim()
    .toLowerCase()
    .replace(/['\u2019]/g, "'")
}

type Token = { kind: 'space' | 'word'; text: string }

const tokens = computed((): Token[] => {
  const raw = props.text ?? ''
  if (!raw) return []
  const out: Token[] = []
  const parts = raw.split(/(\s+)/)
  for (const p of parts) {
    if (!p) continue
    if (/^\s+$/.test(p)) out.push({ kind: 'space', text: p })
    else out.push({ kind: 'word', text: p })
  }
  return out
})

const glossMap = computed(() => {
  const map = new Map<string, string>()
  const src = props.tokenGlosses || {}
  for (const [k, v] of Object.entries(src)) {
    if (typeof v === 'string' && v.trim()) map.set(normalizeTokenKey(k), v.trim())
  }
  return map
})

function glossFor(tokenText: string): string {
  const hit = glossMap.value.get(normalizeTokenKey(tokenText))
  if (hit) return hit
  return (props.phraseGloss || '').trim()
}

function toggle(i: number) {
  openIndex.value = openIndex.value === i ? null : i
}

function close() {
  openIndex.value = null
}

function onDocClick(e: MouseEvent) {
  const root = (e.target as HTMLElement | null)?.closest?.('.lingo-hint-phrase')
  if (!root) close()
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
})
onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
})

watch(
  () => [props.text, props.autoOpenFirst] as const,
  () => {
    openIndex.value = null
    if (!props.autoOpenFirst) return
    const firstWord = tokens.value.findIndex((t) => t.kind === 'word' && glossFor(t.text))
    if (firstWord < 0) return
    openIndex.value = firstWord
    emit('auto-opened')
  },
  { immediate: true }
)
</script>
