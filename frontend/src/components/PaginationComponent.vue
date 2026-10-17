<template>
  <span v-if="hostRef" ref="sentinel" class="feed-pagination-sentinel" aria-hidden="true" />
  <Teleport v-if="!hostRef || mounted" :disabled="!docked" :to="dockTarget">
    <div :class="docked ? 'feed-page__footer' : 'contents'">
      <div :class="docked ? 'feed-page__footer-inner' : 'contents'">
        <div :class="docked ? 'feed-page__pagination' : 'contents'">
          <div
            class="pagination-bar"
            :class="{ 'pagination-bar--end': !showSummary }"
            v-bind="docked ? {} : $attrs"
          >
            <p v-if="showSummary" class="pagination-bar__summary">
              {{ t('pagination.showing') }}
              <span class="font-medium">{{ paginationStart }}</span>
              {{ t('pagination.to') }}
              <span class="font-medium">{{ paginationEnd }}</span>
              {{ t('pagination.of') }} <span class="font-medium">{{ total }}</span>
              {{ t('pagination.items') }}
            </p>

            <div class="pagination-bar__controls">
              <Button
                variant="neutral"
                :disabled="currentPage === 1"
                class="ui-btn--back"
                :class="
                  currentPage === 1
                    ? 'text-gray-400 border-gray-200'
                    : 'text-gray-700 border-gray-300 hover:bg-gray-50'
                "
                @click="go('prev')"
              >
                {{ t('pagination.previous') }}
              </Button>
              <span class="text-sm text-gray-600 whitespace-nowrap">
                {{
                  t('pagination.page', { currentPage: currentPage, totalPages: totalPages || 1 })
                }}
              </span>
              <Button
                variant="neutral"
                :disabled="currentPage >= totalPages"
                class="ui-btn--forward"
                :class="
                  currentPage >= totalPages
                    ? 'text-gray-400 border-gray-200'
                    : 'text-gray-700 border-gray-300 hover:bg-gray-50'
                "
                @click="go('next')"
              >
                {{ t('pagination.next') }}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { feedPaginationHostKey } from '@/composables/useFeedPaginationHost'

defineOptions({ inheritAttrs: false })

const { t } = useI18n()

const emit = defineEmits(['prev', 'next'])

const props = defineProps({
  currentPage: {
    type: Number,
    required: true,
  },
  totalPages: {
    type: Number,
    required: true,
  },
  total: {
    type: Number,
    required: true,
  },
  perPage: {
    type: Number,
    required: true,
  },
})

const hostRef = inject(feedPaginationHostKey, null)
const sentinel = ref<HTMLElement | null>(null)
const mounted = ref(false)
const shown = ref(false)
let observer: MutationObserver | null = null

const paginationStart = computed(() => {
  return (props.currentPage - 1) * props.perPage + 1
})

const paginationEnd = computed(() => {
  return Math.min(props.currentPage * props.perPage, props.total)
})

const showSummary = computed(() => !!(paginationStart.value && paginationEnd.value && props.total))

const docked = computed(() => shown.value && !!hostRef?.value)
const dockTarget = computed(() => hostRef?.value ?? 'body')

function elementShown(el: HTMLElement) {
  if (el.closest('dialog, [role="dialog"]')) return false
  let node: HTMLElement | null = el
  while (node) {
    if (node.hidden) return false
    const style = window.getComputedStyle(node)
    if (style.display === 'none' || style.visibility === 'hidden') return false
    node = node.parentElement
  }
  return true
}

function refresh() {
  const el = sentinel.value
  shown.value = !!(el && hostRef?.value && elementShown(el))
}

function bindObserver() {
  observer?.disconnect()
  const el = sentinel.value
  if (!el) return
  observer = new MutationObserver(() => refresh())
  let node: HTMLElement | null = el
  while (node) {
    observer.observe(node, {
      attributes: true,
      attributeFilter: ['style', 'class', 'hidden'],
    })
    node = node.parentElement
  }
}

function scrollListToTop() {
  const root = sentinel.value?.closest('.feed-page, .discussion-page, .discussion-main')
  const body = root?.querySelector('.feed-page__body') as HTMLElement | null
  body?.scrollTo({ top: 0, behavior: 'instant' })
}

function go(dir: 'prev' | 'next') {
  emit(dir)
  if (docked.value) scrollListToTop()
}

onMounted(() => {
  mounted.value = true
  // The host sits after this component, so its ref is assigned after onMounted.
  nextTick(() => {
    refresh()
    bindObserver()
  })
})

onBeforeUnmount(() => observer?.disconnect())

watch([() => hostRef?.value, sentinel], () => refresh(), { flush: 'post' })
</script>
