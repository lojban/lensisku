<template>
  <span
    :class="[
      config.bg,
      config.text,
      'select-none px-2.5 py-0.5 rounded-full inline-flex items-center gap-1.5',
    ]"
  >
    <img
      v-if="type === 'jbotcan'"
      src="/assets/icons/jbotcan-128.png"
      alt=""
      class="size-3.5 shrink-0 object-contain"
      aria-hidden="true"
    />
    <component :is="config.icon" v-else class="size-3.5 shrink-0" />
    <span>{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { Mail, MessagesSquare } from '@lucide/vue'
import { DefinitionIcon, EntryIcon, WikiIcon } from '@packages/ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  /** One of: 'definition', 'valsi', 'mail', 'jbotcan', 'freeforums', 'wiki' */
  type: {
    type: String,
    required: true,
    validator: (v: unknown) =>
      typeof v === 'string' &&
      ['definition', 'valsi', 'mail', 'jbotcan', 'freeforums', 'wiki'].includes(v),
  },
  /** Override translated label; if not set, uses default for type */
  label: {
    type: String,
    default: '',
  },
})

const { t } = useI18n()

const typeConfig = {
  definition: {
    icon: DefinitionIcon,
    bg: 'definition-chip',
    text: '',
  },
  valsi: {
    icon: EntryIcon,
    bg: 'entry-chip',
    text: '',
  },
  mail: {
    icon: Mail,
    bg: 'mail-chip',
    text: '',
  },
  jbotcan: {
    icon: null,
    bg: 'jbotcan-chip',
    text: '',
  },
  freeforums: {
    icon: MessagesSquare,
    bg: 'freeforums-chip',
    text: '',
  },
  wiki: {
    icon: WikiIcon,
    bg: 'wiki-chip',
    text: '',
  },
}

const defaultLabels = {
  definition: () => t('components.commentItem.inDefinition'),
  valsi: () => t('components.commentItem.inValsi'),
  mail: () => t('home.waveSourceMail'),
  jbotcan: () => 'jbotcan',
  freeforums: () => t('home.waveSourceFreeforums'),
  wiki: () => t('home.waveSourceWiki'),
}

const config = computed(() => typeConfig[props.type as keyof typeof typeConfig])
const label = computed(() =>
  props.label ? props.label : defaultLabels[props.type as keyof typeof defaultLabels]()
)
</script>
