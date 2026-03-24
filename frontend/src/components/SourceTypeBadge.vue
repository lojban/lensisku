<template>
   <span
    :class="[config.bg, config.text, 'px-2.5 py-0.5 rounded-full inline-flex items-center gap-1.5']"
    > <component :is="config.icon" class="size-3.5 shrink-0" /> <span>{{ label }}</span
    > </span
  >
</template>

<script setup lang="ts">
import { BookOpen, BookMarked, Mail } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  /** One of: 'definition', 'valsi', 'mail' */
  type: {
    type: String,
    required: true,
    validator: (v: unknown) =>
      typeof v === 'string' && ['definition', 'valsi', 'mail'].includes(v),
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
    icon: BookOpen,
    bg: 'bg-blue-50',
    text: 'text-blue-700',
  },
  valsi: {
    icon: BookMarked,
    bg: 'bg-emerald-50',
    text: 'text-emerald-700',
  },
  mail: {
    icon: Mail,
    bg: 'bg-violet-50',
    text: 'text-violet-700',
  },
}

const defaultLabels = {
  definition: () => t('components.commentItem.inDefinition'),
  valsi: () => t('components.commentItem.inValsi'),
  mail: () => t('home.waveSourceMail'),
}

const config = computed(() => typeConfig[props.type])
const label = computed(() => (props.label ? props.label : defaultLabels[props.type]()))
</script>

