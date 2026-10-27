<template>
  <div
    class="dialogue-line"
    :class="{
      'dialogue-line--narrator': isNarrator,
      'dialogue-line--right': side === 'right',
      'dialogue-line--playing': playing,
    }"
  >
    <div v-if="!isNarrator" class="dialogue-line__speaker" :data-gender="line.gender || 'x'">
      {{ line.speaker }}
    </div>
    <div class="dialogue-line__bubble">
      <button
        v-if="hasAudio"
        type="button"
        class="dialogue-line__play"
        :aria-label="t('dialogue.playLine')"
        @click="emit('play')"
      >
        <Volume2 class="h-4 w-4" aria-hidden="true" />
      </button>
      <span v-if="showText" class="dialogue-line__text" lang="jbo">{{ line.text }}</span>
      <span v-else class="dialogue-line__hidden">{{ t('dialogue.listen') }}</span>
      <button
        v-if="line.gloss && showText"
        type="button"
        class="dialogue-line__gloss-btn"
        :aria-expanded="glossOpen"
        :aria-label="t('dialogue.showTranslation')"
        @click="toggleGloss"
      >
        <Languages class="h-4 w-4" aria-hidden="true" />
      </button>
      <div v-if="glossOpen && line.gloss" class="dialogue-line__gloss">{{ line.gloss }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Languages, Volume2 } from '@lucide/vue'
import { NARRATOR, type DialogueLine } from './types'

const props = defineProps<{
  line: DialogueLine
  side: 'left' | 'right'
  showText: boolean
  hasAudio: boolean
  playing: boolean
}>()
const emit = defineEmits<{ play: []; gloss: [] }>()
const { t } = useI18n()

const isNarrator = computed(() => props.line.speaker === NARRATOR)
const glossOpen = ref(false)

function toggleGloss() {
  glossOpen.value = !glossOpen.value
  if (glossOpen.value) emit('gloss')
}
</script>

<style scoped>
.dialogue-line {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.15rem;
  animation: dialogue-line-in 0.25s ease-out;
}
.dialogue-line--right {
  align-items: flex-end;
}
.dialogue-line--narrator {
  align-items: center;
}
.dialogue-line__speaker {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #6b7280;
}
.dialogue-line__speaker[data-gender='f'] {
  color: #be185d;
}
.dialogue-line__speaker[data-gender='m'] {
  color: #1d4ed8;
}
.dialogue-line__bubble {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
  max-width: 34rem;
  padding: 0.55rem 0.8rem;
  border: 2px solid #e5e7eb;
  border-radius: 1rem;
  background: #fff;
  font-size: 1rem;
  line-height: 1.4;
  overflow-wrap: anywhere;
}
.dialogue-line--narrator .dialogue-line__bubble {
  border-style: dashed;
  background: #f9fafb;
  font-style: italic;
}
.dialogue-line--playing .dialogue-line__bubble {
  border-color: #58cc02;
}
.dialogue-line__play,
.dialogue-line__gloss-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 9999px;
  color: #1cb0f6;
  flex-shrink: 0;
}
.dialogue-line__play:hover,
.dialogue-line__gloss-btn:hover {
  background: #eff6ff;
}
.dialogue-line__hidden {
  color: #9ca3af;
  font-style: italic;
}
.dialogue-line__gloss {
  flex-basis: 100%;
  font-size: 0.85rem;
  color: #4b5563;
  border-top: 1px dashed #e5e7eb;
  padding-top: 0.3rem;
}
@keyframes dialogue-line-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
}
</style>
