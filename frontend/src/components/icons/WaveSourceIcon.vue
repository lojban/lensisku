<template>
  <img
    v-if="source === 'jbotcan'"
    src="/assets/icons/jbotcan-128.png"
    alt=""
    class="h-4 w-4 shrink-0 object-contain"
    aria-hidden="true"
  />
  <component :is="lucideIcon" v-else class="h-4 w-4 shrink-0" aria-hidden="true" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { BookA, Layers, Mail, MessageSquare, MessagesSquare } from '@lucide/vue'

const props = defineProps<{
  source: 'all' | 'jbotcan' | 'freeforums' | 'comments' | 'mail' | 'wiki'
}>()

const lucideBySource = {
  all: Layers,
  freeforums: MessagesSquare,
  comments: MessageSquare,
  mail: Mail,
  wiki: BookA,
} as const

const lucideIcon = computed(() =>
  props.source === 'jbotcan' ? null : lucideBySource[props.source]
)
</script>
