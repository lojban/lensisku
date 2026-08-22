<template>
  <div
    class="semantic-graph-root flex min-h-0 min-w-0 w-full flex-1 flex-col gap-3 px-2 pb-3 pt-2 md:px-4"
  >
    <div class="shrink-0 space-y-3">
      <h1 class="text-lg font-semibold text-gray-900 md:text-xl">
        {{ t('semanticGraph.title') }}
      </h1>

      <p class="text-sm text-gray-600">{{ t('semanticGraph.description') }}</p>

      <form class="flex flex-col gap-2 sm:flex-row sm:items-center" @submit.prevent="onSearchEnter">
        <div class="sg-search-group flex min-w-0 flex-1 items-center gap-2">
          <Input
            id="semantic-graph-search"
            v-model="searchQuery"
            type="search"
            class="input-field sg-search-input w-full min-w-0 max-w-full"
            :placeholder="t('semanticGraph.anchorPlaceholder')"
            :aria-label="t('semanticGraph.anchorPlaceholder')"
            autocomplete="off"
          />
          <Button
            type="button"
            variant="ui-btn--empty"
            :disabled="isResetDisabled"
            :title="t('semanticGraph.reset')"
            @click="onResetClick"
          >
            <template #icon> <RotateCcw :size="14" aria-hidden="true" /> </template>
            {{ t('semanticGraph.reset') }}
          </Button>
        </div>

        <div
          class="btn-group flex items-stretch"
          role="group"
          :aria-label="t('semanticGraph.ioGroupLabel')"
        >
          <Button
            type="button"
            variant="read"
            :disabled="!cyReady"
            :title="t('semanticGraph.export')"
            @click="exportGraphFile"
          >
            <template #icon> <ExportIcon :size="14" aria-hidden="true" /> </template>
            {{ t('semanticGraph.export') }}
          </Button>
          <Button
            type="button"
            variant="insert"
            y
            :title="t('semanticGraph.import')"
            @click="triggerImport"
          >
            <template #icon> <ImportIcon :size="14" aria-hidden="true" /> </template>
            {{ t('semanticGraph.import') }}
          </Button>
          <FileInput
            ref="importInputRef"
            type="file"
            accept="application/json,.json"
            class="sr-only"
            @change="onImportFile"
          />
        </div>
      </form>
      <CombinedFilters
        v-model="combinedFiltersModel"
        v-model:graph-build-params="graphBuildParams"
        :languages="languages"
        languages-in-expanded-panel
        hide-semantic-graph-metrics
        :show-collection-filter="false"
        :semantic-graph-max-nodes="GRAPH_METRICS_MAX"
        class="w-full"
      />
      <div
        v-if="graphError"
        class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
        role="alert"
      >
        {{ graphError }}
      </div>
    </div>

    <div
      ref="cyContainerRef"
      class="cy-shell min-h-[min(60dvh,22rem)] w-full shrink-0 rounded-lg border border-gray-200 bg-gradient-to-br from-slate-50 via-white to-slate-100/90 sm:min-h-[min(50vh,28rem)] md:min-h-[min(55vh,34rem)]"
      aria-label="Semantic similarity graph"
    />

    <div
      v-if="graphBuildParams"
      class="shrink-0 grid grid-cols-2 gap-3 rounded-lg bg-white p-3 shadow-sm sm:grid-cols-3 md:grid-cols-5"
    >
      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-min-vote">{{
          t('semanticGraph.minVote')
        }}</label>
        <Input
          id="cf-sg-min-vote"
          v-model.number="graphBuildParams.minVote"
          type="number"
          class="input-field h-8 w-full"
          min="-999"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-limit">{{
          t('semanticGraph.nodeLimit')
        }}</label>
        <Input
          id="cf-sg-limit"
          v-model.number="graphBuildParams.graphLimit"
          type="number"
          class="input-field h-8 w-full"
          min="1"
          :max="GRAPH_METRICS_MAX"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-k">{{ t('semanticGraph.kNeighbors') }}</label>
        <Input
          id="cf-sg-k"
          v-model.number="graphBuildParams.kNeighbors"
          type="number"
          class="input-field h-8 w-full"
          min="1"
          max="30"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-min-sim">{{
          t('semanticGraph.minPairwiseSim')
        }}</label>
        <Input
          id="cf-sg-min-sim"
          v-model.number="graphBuildParams.minPairwiseSim"
          type="number"
          class="input-field h-8 w-full"
          min="0"
          max="1"
          step="0.05"
        />
      </div>
    </div>
    <ModalComponent :show="!!previewDef" :title="previewModalTitle" @close="previewDef = null">
      <p v-if="previewDef?.type_name" class="mb-2 text-xs text-gray-500">
        {{ previewDef.type_name }}
      </p>

      <div class="text-sm prose prose-sm max-w-none text-gray-700">
        <LazyMathJax :content="previewDef?.definition ?? ''" />
      </div>
      <a
        v-if="previewEntryHref"
        class="mt-4 inline-flex text-sm font-medium text-nav-link underline"
        :href="previewEntryHref"
        target="_blank"
        rel="noopener noreferrer"
      >
        {{ t('semanticGraph.openEntry') }}
      </a>
    </ModalComponent>
    <div
      v-if="graphLoading"
      class="pointer-events-none fixed inset-0 z-30 flex items-center justify-center bg-white/40"
      aria-busy="true"
      :aria-label="t('semanticGraph.loading')"
    >
      <div
        class="h-10 w-10 animate-spin rounded-full border-2 border-cornflower-500 border-t-transparent"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, FileInput, Input, ExportIcon, ImportIcon } from '@packages/ui'
import { RotateCcw } from '@lucide/vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import type { Core, ElementDefinition, LayoutOptions } from 'cytoscape'

import CombinedFilters, { type SemanticGraphBuildParams } from '@/components/CombinedFilters.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { fetchSemanticGraph, getDefinition, getLanguages } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'
import {
  combinedFiltersFromQuery,
  combinedFiltersToQuery,
  compactQuery,
} from '@/utils/routeQuery'
import type { SupportedLocale } from '@/router'

const { t, locale } = useI18n()
const route = useRoute()
const router = useRouter()

/** Must match server `SEMANTIC_GRAPH_MAX_LIMIT` (anchor fetch pool cap). */
const GRAPH_METRICS_MAX = 200

type SemanticGraphApiPayload = {
  nodes: Array<{
    id: string
    definitionid: number
    label: string
    word: string
    query_similarity?: number | null
  }>
  edges: Array<{ source: string; target: string; similarity: number }>
}

useSeoHead({
  title: computed(() => t('semanticGraph.title')),
  pathWithoutLocale: '/semantic-graph',
})

function querySearchToString(q: unknown): string {
  if (typeof q === 'string') return q
  if (Array.isArray(q) && q.length > 0 && typeof q[0] === 'string') return q[0]
  return ''
}

const searchQuery = ref(normalizeSearchQuery(querySearchToString(route.query.search)) as string)
const languages = ref<
  Array<{ id: number; real_name: string; english_name: string; tag: string; lojban_name?: string }>
>([])

const urlFilters = combinedFiltersFromQuery(route.query)
const combinedFiltersModel = ref({
  ...urlFilters,
  isSemantic: querySearchToString(route.query.mode) !== 'dictionary',
})

const graphBuildParams = ref<SemanticGraphBuildParams>({
  minVote: 1,
  graphLimit: 80,
  kNeighbors: 6,
  minPairwiseSim: 0.15,
})

const cyContainerRef = ref<HTMLDivElement | null>(null)
const importInputRef = ref<HTMLInputElement | null>(null)
const cyReady = ref(false)
let cy: Core | null = null

const graphLoading = ref(false)
const graphError = ref<string | null>(null)

const graphMode = ref<'preview' | 'anchor'>('preview')
const anchorNodeId = ref<string | null>(null)

/**
 * Zoom level when the current LOD mode was entered (overview fit, or focus-neighborhood fit).
 * Compared only within the same mode — never across overview↔focus swaps.
 */
let modeEntryZoom = 1
/** Last focus valsi fetched for a zoom neighborhood (skip duplicate requests). */
let activeFocusWord: string | null = null
/** In-flight graph request id (drop stale responses). */
let graphFetchSeq = 0
let graphRenderInProgress = false
/** Blocks zoom-triggered refetches while layout/fit runs; queues one follow-up. */
let suppressZoomRefetch = false
let pendingViewportRefetch = false
let zoomRefetchTimer: ReturnType<typeof setTimeout> | null = null
let suppressClearTimer: ReturnType<typeof setTimeout> | null = null
let cyZoomPanHandler: (() => void) | null = null

/** Zoom-in past this factor of mode-entry zoom → leave overview for a focus neighborhood. */
const ZOOM_IN_FACTOR = 1.08
/** Zoom-out below this factor of mode-entry zoom → leave focus and restore overview. */
const ZOOM_OUT_FACTOR = 0.78
const ZOOM_REFETCH_DEBOUNCE_MS = 200

type PreviewDefinition = {
  valsiword?: string
  word?: string
  type_name?: string
  definition?: string
  langid?: number
}

const previewDef = ref<PreviewDefinition | null>(null)

const previewModalTitle = computed(() => {
  const d = previewDef.value
  if (!d) return ''
  return (d.valsiword ?? d.word ?? '').trim()
})

const previewEntryHref = computed(() => {
  const d = previewDef.value
  if (!d) return ''
  const w = (d.valsiword ?? d.word ?? '').replace(/ /g, '_')
  if (!w) return ''
  const langid = d.langid
  return router.resolve({
    name: `Entry-${locale.value as SupportedLocale}`,
    params: { id: w },
    ...(langid != null ? { query: { langid: String(langid) } } : {}),
  }).href
})

function buildGraphParams(opts?: {
  preview?: boolean
  focus?: string
}): Record<string, unknown> {
  const f = combinedFiltersModel.value
  const preview = opts?.preview === true
  const g = graphBuildParams.value
  const focus = opts?.focus?.trim() || ''
  const params: Record<string, unknown> = {
    min_vote: g.minVote,
    limit: Math.max(1, Math.min(GRAPH_METRICS_MAX, g.graphLimit)),
    k_neighbors: g.kNeighbors,
    min_similarity: g.minPairwiseSim,
  }
  if (focus) {
    params.focus = focus
  } else if (preview) {
    params.preview = true
  } else {
    params.search = normalizeSearchQuery(searchQuery.value).trim()
  }
  if (f.selectedLanguages?.length) {
    params.languages = f.selectedLanguages.join(',')
  }
  if (f.selmaho) params.selmaho = f.selmaho
  if (f.usernames?.length) params.username = f.usernames.join(',')
  if (f.excludeUsernames?.length) params.exclude_usernames = f.excludeUsernames.join(',')
  if (f.word_type) params.word_type = f.word_type
  if (f.source_langid && f.source_langid !== 1) {
    params.source_langid = f.source_langid
  }
  if (f.searchInPhrases === false) {
    params.search_in_phrases = false
  }
  if (!preview && !focus && f.isSemantic === false) {
    params.semantic = false
  }
  return params
}

function detachCyZoomHandlers() {
  if (cy && cyZoomPanHandler) {
    cy.off('zoom', cyZoomPanHandler)
    cy.off('pan', cyZoomPanHandler)
    cyZoomPanHandler = null
  }
}

function clearZoomRefetch() {
  if (zoomRefetchTimer != null) {
    clearTimeout(zoomRefetchTimer)
    zoomRefetchTimer = null
  }
  if (suppressClearTimer != null) {
    clearTimeout(suppressClearTimer)
    suppressClearTimer = null
  }
  detachCyZoomHandlers()
  anchorNodeId.value = null
  activeFocusWord = null
  modeEntryZoom = 1
  pendingViewportRefetch = false
  suppressZoomRefetch = false
}

function attachViewportHandlers(core: Core) {
  detachCyZoomHandlers()
  cyZoomPanHandler = () => {
    if (!cy) return
    applyZoomStabilizedNodeSizes(cy)
    scheduleViewportRefetch()
  }
  core.on('zoom', cyZoomPanHandler)
  core.on('pan', cyZoomPanHandler)
}

function beginZoomSuppress() {
  suppressZoomRefetch = true
  if (suppressClearTimer != null) {
    clearTimeout(suppressClearTimer)
    suppressClearTimer = null
  }
}

function endZoomSuppress() {
  suppressZoomRefetch = false
  if (cy) modeEntryZoom = cy.zoom() || 1
  if (pendingViewportRefetch) {
    pendingViewportRefetch = false
    scheduleViewportRefetch()
  }
}

/** Valsi (word) on the node closest to the current viewport center. */
function wordAtViewportCenter(core: Core): string | null {
  const ext = core.extent()
  const cx = (ext.x1 + ext.x2) / 2
  const cy_ = (ext.y1 + ext.y2) / 2
  let best: { word: string; d2: number } | null = null
  core.nodes().forEach((n) => {
    const p = n.position()
    const dx = p.x - cx
    const dy = p.y - cy_
    const d2 = dx * dx + dy * dy
    const word = String(n.data('word') ?? '').trim()
    if (!word) return
    if (!best || d2 < best.d2) best = { word, d2 }
  })
  return best?.word ?? null
}

function applyZoomStabilizedNodeSizes(core: Core) {
  const z = Math.max(0.12, Math.min(core.zoom(), 6))
  const inv = 1 / z
  const base = 28 * inv
  const anchorBase = 78 * inv
  const fs = 10 * inv
  const afs = 14 * inv
  const anchorBorder = 5 * inv
  core
    .style()
    .selector('node')
    .style({ width: base, height: base, 'font-size': fs })
    .selector('node[isAnchor = "yes"]')
    .style({
      width: anchorBase,
      height: anchorBase,
      'font-size': afs,
      'font-weight': 700,
      'border-width': anchorBorder,
    })
    .update()
}

async function restoreBaseGraph(seq: number) {
  if (graphMode.value === 'preview' || !normalizeSearchQuery(searchQuery.value).trim()) {
    graphMode.value = 'preview'
    const res = await fetchSemanticGraph(buildGraphParams({ preview: true }))
    if (seq !== graphFetchSeq || !cy) return
    await renderGraph(res.data, 'preview')
  } else {
    const q = normalizeSearchQuery(searchQuery.value).trim()
    const res = await fetchSemanticGraph(buildGraphParams({ preview: false }))
    if (seq !== graphFetchSeq || !cy) return
    await renderGraph(res.data, 'anchor', q)
  }
}

/** After zoom/pan settles: neighborhood of the center word, or restore overview when zoomed out. */
async function refetchForViewport() {
  if (!cy || graphRenderInProgress) return
  if (suppressZoomRefetch) {
    pendingViewportRefetch = true
    return
  }

  const z = cy.zoom() || 1
  const entry = modeEntryZoom || 1
  const focus = wordAtViewportCenter(cy)

  // --- Focus mode: zoom out → overview; pan/zoom to a new center → new neighborhood ---
  if (activeFocusWord != null) {
    if (z <= entry * ZOOM_OUT_FACTOR) {
      activeFocusWord = null
      const seq = ++graphFetchSeq
      graphRenderInProgress = true
      graphLoading.value = true
      try {
        await restoreBaseGraph(seq)
      } catch {
        /* keep previous graph */
      } finally {
        if (seq === graphFetchSeq) graphLoading.value = false
        graphRenderInProgress = false
      }
      return
    }

    if (!focus) return
    if (activeFocusWord.toLowerCase() === focus.toLowerCase()) return

    const seq = ++graphFetchSeq
    graphRenderInProgress = true
    activeFocusWord = focus
    graphLoading.value = true
    try {
      const res = await fetchSemanticGraph(buildGraphParams({ focus }))
      if (seq !== graphFetchSeq || !cy) return
      await renderGraph(res.data, 'anchor', focus)
    } catch (e: unknown) {
      activeFocusWord = null
      const err = e as { response?: { data?: { error?: string } } }
      const msg = err.response?.data?.error
      if (typeof msg === 'string' && msg.trim()) graphError.value = msg
    } finally {
      if (seq === graphFetchSeq) graphLoading.value = false
      graphRenderInProgress = false
    }
    return
  }

  // --- Overview mode: zoom in past threshold → focus neighborhood of center word ---
  if (z < entry * ZOOM_IN_FACTOR) return
  if (!focus) return

  const seq = ++graphFetchSeq
  graphRenderInProgress = true
  activeFocusWord = focus
  graphLoading.value = true
  try {
    const res = await fetchSemanticGraph(buildGraphParams({ focus }))
    if (seq !== graphFetchSeq || !cy) return
    await renderGraph(res.data, 'anchor', focus)
  } catch (e: unknown) {
    activeFocusWord = null
    const err = e as { response?: { data?: { error?: string } } }
    const msg = err.response?.data?.error
    if (typeof msg === 'string' && msg.trim()) graphError.value = msg
  } finally {
    if (seq === graphFetchSeq) graphLoading.value = false
    graphRenderInProgress = false
  }
}

function scheduleViewportRefetch() {
  if (!cy) return
  if (suppressZoomRefetch) {
    pendingViewportRefetch = true
    return
  }
  if (zoomRefetchTimer != null) clearTimeout(zoomRefetchTimer)
  zoomRefetchTimer = setTimeout(() => {
    zoomRefetchTimer = null
    void refetchForViewport()
  }, ZOOM_REFETCH_DEBOUNCE_MS)
}

async function loadPreviewGraph() {
  if (!cyReady.value || !cy) {
    graphError.value = t('semanticGraph.errorNotReady')
    return
  }
  graphError.value = null
  graphLoading.value = true
  try {
    clearZoomRefetch()
    graphMode.value = 'preview'
    graphFetchSeq++
    const res = await fetchSemanticGraph(buildGraphParams({ preview: true }))
    await renderGraph(res.data, 'preview')
  } catch (e: unknown) {
    const err = e as { response?: { data?: { error?: string }; status?: number } }
    const msg =
      err.response?.data?.error ??
      (err.response?.status === 503 ? t('semanticGraph.errorDisabled') : null) ??
      t('semanticGraph.errorLoad')
    graphError.value = typeof msg === 'string' ? msg : t('semanticGraph.errorLoad')
  } finally {
    graphLoading.value = false
  }
}

async function buildGraph() {
  if (!cyReady.value || !cy) {
    graphError.value = t('semanticGraph.errorNotReady')
    return
  }
  const q = normalizeSearchQuery(searchQuery.value).trim()
  if (!q) {
    graphError.value = t('semanticGraph.errorNeedSearch')
    return
  }
  graphError.value = null
  graphLoading.value = true
  try {
    clearZoomRefetch()
    graphMode.value = 'anchor'
    const seq = ++graphFetchSeq
    graphRenderInProgress = true
    const res = await fetchSemanticGraph(buildGraphParams({ preview: false }))
    if (seq !== graphFetchSeq) return
    await renderGraph(res.data, 'anchor', q)
  } catch (e: unknown) {
    const err = e as { response?: { data?: { error?: string }; status?: number } }
    const msg =
      err.response?.data?.error ??
      (err.response?.status === 503 ? t('semanticGraph.errorDisabled') : null) ??
      t('semanticGraph.errorLoad')
    graphError.value = typeof msg === 'string' ? msg : t('semanticGraph.errorLoad')
  } finally {
    graphLoading.value = false
    graphRenderInProgress = false
  }
}

const SEARCH_DEBOUNCE_MS = 450

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

function cancelSearchDebounce() {
  if (searchDebounceTimer != null) {
    clearTimeout(searchDebounceTimer)
    searchDebounceTimer = null
  }
}

async function runDebouncedSearchBuild() {
  if (!cyReady.value || !cy) return
  const q = normalizeSearchQuery(searchQuery.value).trim()
  if (!q) {
    graphError.value = null
    await loadPreviewGraph()
    return
  }
  await buildGraph()
}

function scheduleSearchBuild() {
  cancelSearchDebounce()
  searchDebounceTimer = setTimeout(() => {
    searchDebounceTimer = null
    void runDebouncedSearchBuild()
    syncSearchQueryToRoute()
  }, SEARCH_DEBOUNCE_MS)
}

function syncSearchQueryToRoute() {
  const q = normalizeSearchQuery(searchQuery.value).trim()
  const fromRoute = normalizeSearchQuery(querySearchToString(route.query.search)).trim()
  if (q === fromRoute) return
  const nextQuery = { ...route.query } as Record<string, string | string[] | undefined>
  if (q) nextQuery.search = q
  else delete nextQuery.search
  void router.replace({ path: route.path, query: nextQuery })
}

watch(
  combinedFiltersModel,
  () => {
    const next = compactQuery({
      ...route.query,
      ...combinedFiltersToQuery({
        ...combinedFiltersModel.value,
        selectedCollections: combinedFiltersModel.value.selectedCollections ?? [],
      }),
      mode: combinedFiltersModel.value.isSemantic === false ? 'dictionary' : 'semantic',
    })
    const current = compactQuery({ ...route.query })
    if (JSON.stringify(next) === JSON.stringify(current)) return
    void router.replace({ path: route.path, query: next })
  },
  { deep: true }
)

watch(searchQuery, () => {
  scheduleSearchBuild()
})

watch(
  () => normalizeSearchQuery(querySearchToString(route.query.search)).trim(),
  (fromRouteTrimmed) => {
    const localTrimmed = normalizeSearchQuery(searchQuery.value).trim()
    if (fromRouteTrimmed === localTrimmed) return
    searchQuery.value = fromRouteTrimmed
    cancelSearchDebounce()
    scheduleSearchBuild()
  }
)

function onSearchEnter() {
  cancelSearchDebounce()
  void buildGraph()
  syncSearchQueryToRoute()
}

async function onResetClick() {
  searchQuery.value = ''
  await nextTick()
  cancelSearchDebounce()
  syncSearchQueryToRoute()
  await loadPreviewGraph()
}

const isResetDisabled = computed(() => graphMode.value === 'preview' && !searchQuery.value.trim())

/** FNV-1a 32-bit — stable, fast hash for palette picking from valsi text. */
function hashString32(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

function hslToHex(h: number, s: number, l: number): string {
  const hue = ((h % 360) + 360) % 360
  const sat = Math.max(0, Math.min(100, s)) / 100
  const light = Math.max(0, Math.min(100, l)) / 100
  const c = (1 - Math.abs(2 * light - 1)) * sat
  const hp = hue / 60
  const x = c * (1 - Math.abs((hp % 2) - 1))
  let r1 = 0
  let g1 = 0
  let b1 = 0
  if (hp < 1) {
    r1 = c
    g1 = x
  } else if (hp < 2) {
    r1 = x
    g1 = c
  } else if (hp < 3) {
    g1 = c
    b1 = x
  } else if (hp < 4) {
    g1 = x
    b1 = c
  } else if (hp < 5) {
    r1 = x
    b1 = c
  } else {
    r1 = c
    b1 = x
  }
  const m = light - c / 2
  const r = Math.round((r1 + m) * 255)
  const g = Math.round((g1 + m) * 255)
  const b = Math.round((b1 + m) * 255)
  const hex = (n: number) => n.toString(16).padStart(2, '0')
  return `#${hex(r)}${hex(g)}${hex(b)}`
}

/** Saturated but readable fills; darker borders for separation on light canvas. */
function nodeColorsFromValsiKey(key: string): { bgColor: string; borderColor: string } {
  const k = key.trim().toLowerCase() || '·'
  const h = hashString32(k)
  const hue = h % 360
  const sat = 52 + ((h >>> 8) % 22)
  const light = 54 + ((h >>> 16) % 12)
  const bgColor = hslToHex(hue, sat, light)
  const borderColor = hslToHex(hue, Math.min(88, sat + 10), Math.max(28, light - 22))
  return { bgColor, borderColor }
}

function ensureNodePaletteColors(core: Core) {
  core.nodes().forEach((n) => {
    if (n.data('bgColor')) return
    const key = String(n.data('word') ?? n.data('label') ?? n.data('id') ?? '').trim()
    const { bgColor, borderColor } = nodeColorsFromValsiKey(key || '·')
    n.data('bgColor', bgColor)
    n.data('borderColor', borderColor)
  })
}

/** Exact case-insensitive `word` match wins; otherwise trust backend ordering (first = best). */
function pickAnchorNodeId(
  nodes: SemanticGraphApiPayload['nodes'],
  searchTrim: string
): string | null {
  if (nodes.length === 0) return null
  const q = searchTrim.trim().toLowerCase()
  if (q) {
    for (const n of nodes) {
      const w = (n.word || '').trim().toLowerCase()
      if (w && w === q) return n.id
    }
    for (const n of nodes) {
      const w = (n.word || '').trim().toLowerCase()
      if (w && (w.startsWith(`${q} `) || w.startsWith(`${q}-`))) return n.id
    }
  }
  return nodes[0].id
}

function elementsFromApi(data: SemanticGraphApiPayload, anchorId: string | null) {
  const nodes = data.nodes.map((n) => {
    const valsiKey = (n.word || n.label || n.id || '').trim()
    const displayLabel =
      (n.word || '').trim() || (n.label || '').split(/\s*·\s*/)[0]?.trim() || n.id
    const { bgColor, borderColor } = nodeColorsFromValsiKey(valsiKey)
    const isAnchor = anchorId != null && n.id === anchorId ? 'yes' : 'no'
    return {
      data: {
        id: n.id,
        label: displayLabel,
        definitionid: n.definitionid,
        word: n.word,
        qs: n.query_similarity ?? undefined,
        bgColor,
        borderColor,
        isAnchor,
      },
    }
  })
  const edges = data.edges.map((e, i) => {
    const touchAnchor = anchorId != null && (e.source === anchorId || e.target === anchorId)
    return {
      data: {
        id: `e${i}-${e.source}-${e.target}`,
        source: e.source,
        target: e.target,
        similarity: e.similarity,
        isAnchorEdge: touchAnchor ? 'yes' : 'no',
      },
    }
  })
  return [...nodes, ...edges]
}

async function renderGraph(
  apiData: SemanticGraphApiPayload,
  mode: 'preview' | 'anchor',
  anchorSearchTrim = ''
) {
  if (!cy || !cyContainerRef.value) return
  detachCyZoomHandlers()
  cy.elements().remove()
  const anchorId = mode === 'anchor' ? pickAnchorNodeId(apiData.nodes, anchorSearchTrim) : null
  anchorNodeId.value = anchorId
  const els = elementsFromApi(apiData, anchorId)
  if (els.length === 0) {
    // Keep interaction alive even on empty responses.
    attachViewportHandlers(cy)
    endZoomSuppress()
    return
  }
  cy.add(els)
  ensureNodePaletteColors(cy)

  beginZoomSuppress()

  let finished = false
  const finishLayout = () => {
    if (finished || !cy) return
    finished = true
    cy.fit(undefined, 32)
    modeEntryZoom = cy.zoom() || 1
    applyZoomStabilizedNodeSizes(cy)
    attachViewportHandlers(cy)
    // Layout animation / fit can still emit zoom briefly — clear suppress after a short settle.
    if (suppressClearTimer != null) clearTimeout(suppressClearTimer)
    suppressClearTimer = setTimeout(() => {
      suppressClearTimer = null
      endZoomSuppress()
    }, 120)
  }

  // fcose/layoutstop can fail to fire in edge cases — never leave suppress stuck on.
  if (suppressClearTimer != null) clearTimeout(suppressClearTimer)
  suppressClearTimer = setTimeout(() => {
    finishLayout()
  }, 2000)

  try {
    if (mode === 'anchor' && anchorId) {
      const layoutOpts = {
        name: 'fcose',
        quality: 'proof',
        randomize: true,
        animate: true,
        animationDuration: 450,
        fit: false,
        padding: 32,
        nodeRepulsion: () => 18000,
        idealEdgeLength: (edge: { data: (k: string) => unknown }) => {
          const s = Number(edge.data('similarity')) || 0
          const isAnchorEdge = edge.data('isAnchorEdge') === 'yes'
          const base = 70 + 250 * (1 - Math.max(0, Math.min(1, s)))
          return isAnchorEdge ? base * 0.78 : base
        },
        edgeElasticity: 0.2,
        gravity: 0.25,
        gravityRangeCompound: 1.5,
        nodeSeparation: 90,
        nestingFactor: 0.9,
        numIter: 3500,
        tile: false,
        fixedNodeConstraint: [{ nodeId: anchorId, position: { x: 0, y: 0 } }],
      } as unknown as LayoutOptions
      const layout = cy.layout(layoutOpts)
      layout.one('layoutstop', finishLayout)
      layout.run()
    } else {
      const layout = cy.layout({
        name: 'fcose',
        quality: 'default',
        randomize: true,
        animate: true,
        animationDuration: 350,
        fit: false,
        padding: 16,
      } as LayoutOptions)
      layout.one('layoutstop', finishLayout)
      layout.run()
    }
  } catch {
    finishLayout()
  }
}

async function initCy() {
  const container = cyContainerRef.value
  if (!container) return
  const cytoscape = (await import('cytoscape')).default
  const fcose = (await import('cytoscape-fcose')).default
  cytoscape.use(fcose)

  cy = cytoscape({
    container,
    wheelSensitivity: 0.45,
    minZoom: 0.05,
    maxZoom: 10,
    userZoomingEnabled: true,
    userPanningEnabled: true,
    style: [
      {
        selector: 'node',
        style: {
          'background-color': 'data(bgColor)',
          'border-color': 'data(borderColor)',
          'border-width': 2,
          shape: 'ellipse',
          label: 'data(label)',
          'font-size': '10px',
          'font-weight': 500,
          'text-wrap': 'wrap',
          'text-max-width': '80px',
          color: '#0f172a',
          'text-outline-width': 3,
          'text-outline-color': '#ffffff',
          'text-outline-opacity': 0.95,
          width: 26,
          height: 26,
        },
      },
      {
        selector: 'edge',
        style: {
          'curve-style': 'bezier',
          'line-color': '#c7d2e0',
          opacity: 0.88,
          width: 'mapData(similarity, 0.15, 1, 1, 4.5)',
        },
      },
      {
        selector: 'edge[isAnchorEdge = "yes"]',
        style: {
          'line-color': '#94a3b8',
          opacity: 0.95,
          width: 'mapData(similarity, 0.15, 1, 2.5, 7)',
        },
      },
      {
        selector: 'node:selected',
        style: {
          'border-width': 3,
          'border-color': '#3D6BC4',
          'background-color': '#5789E8',
        },
      },
      {
        selector: 'node[isAnchor = "yes"]',
        style: {
          'border-width': 4,
          'border-color': '#15803d',
        },
      },
    ],
    elements: [],
  })

  cy.on('tap', 'node', async (evt) => {
    const n = evt.target
    const defId = n.data('definitionid') as number | undefined
    if (defId == null) return
    previewDef.value = { valsiword: n.data('word'), definition: '', langid: undefined }
    try {
      const res = await getDefinition(defId)
      previewDef.value = res.data
    } catch {
      graphError.value = t('semanticGraph.errorPreview')
    }
  })

  cy.on('tap', (evt) => {
    if (evt.target === cy) {
      previewDef.value = null
    }
  })

  cyReady.value = true
}

function exportGraphFile() {
  if (!cy) return
  const payload = {
    v: 1,
    elements: cy.elements().map((el) => el.json()),
    zoom: cy.zoom(),
    pan: cy.pan(),
  }
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' })
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = 'semantic-graph.json'
  a.click()
  URL.revokeObjectURL(a.href)
}

function triggerImport() {
  importInputRef.value?.click()
}

function onImportFile(ev: Event) {
  const input = ev.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || !cy) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const parsed = JSON.parse(String(reader.result)) as {
        v?: number
        elements?: ElementDefinition[]
        zoom?: number
        pan?: { x: number; y: number }
      }
      if (!parsed.elements?.length) return
      clearZoomRefetch()
      graphMode.value = 'preview'
      cy!.elements().remove()
      cy!.add(parsed.elements as ElementDefinition[])
      ensureNodePaletteColors(cy!)
      cy!.layout({ name: 'preset', fit: true, padding: 16 }).run()
      if (typeof parsed.zoom === 'number') cy!.zoom(parsed.zoom)
      if (parsed.pan && typeof parsed.pan.x === 'number' && typeof parsed.pan.y === 'number') {
        cy!.pan(parsed.pan)
      }
    } catch {
      graphError.value = t('semanticGraph.errorImport')
    }
  }
  reader.readAsText(file)
}

onMounted(async () => {
  try {
    const langRes = await getLanguages()
    languages.value = langRes.data
  } catch {
    graphError.value = t('semanticGraph.errorLanguages')
  }
  await initCy()
  const initialQ = normalizeSearchQuery(searchQuery.value).trim()
  if (initialQ) {
    await buildGraph()
  } else {
    await loadPreviewGraph()
  }
})

onBeforeUnmount(() => {
  cancelSearchDebounce()
  clearZoomRefetch()
  if (cy) {
    cy.destroy()
    cy = null
  }
  cyReady.value = false
})
</script>

<style scoped>
.cy-shell :deep(canvas[data-id='layer0-selectbox']) {
  left: unset !important;
  top: unset !important;
}

.cy-shell :deep(canvas) {
  border-radius: 0.5rem;
}

.semantic-graph-root {
  position: relative;
}

.sg-search-input::-webkit-search-cancel-button,
.sg-search-input::-webkit-search-decoration {
  -webkit-appearance: none;
  appearance: none;
}

/** Join Export/Import as a forced segmented pair while preserving each button's semantic palette
 *  (brandbook §6.3: distinct roles `read` vs `insert`; stock `.btn-group-forced` rule only targets
 *  `.ui-btn--group-item`, so geometry is applied locally to non-group-item segments). */
.sg-io-group .sg-io-segment {
  position: relative;
  border-radius: 0;
  margin-left: -1px;
}
.sg-io-group .sg-io-segment:first-child {
  margin-left: 0;
  border-top-left-radius: 9999px;
  border-bottom-left-radius: 9999px;
}
.sg-io-group .sg-io-segment:last-of-type {
  border-top-right-radius: 9999px;
  border-bottom-right-radius: 9999px;
}
.sg-io-group .sg-io-segment:hover,
.sg-io-group .sg-io-segment:focus-visible {
  z-index: 1;
}
</style>
