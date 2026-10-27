<template>
  <div class="dialogue-q">
    <div class="dialogue-q__label">
      {{ t('dialogue.questionOf', { n: question.index + 1, total }) }}
    </div>
    <p v-if="question.prompt" class="dialogue-q__prompt">{{ question.prompt }}</p>

    <!-- Choice -->
    <div v-if="question.kind === 'choice'" class="dialogue-q__options">
      <button
        v-for="(opt, i) in question.options"
        :key="i"
        type="button"
        class="dialogue-q__option"
        :class="optionClass(opt.text)"
        :disabled="answered || busy"
        lang="jbo"
        @click="picked = opt.text"
      >
        {{ opt.text }}
      </button>
    </div>

    <!-- Fill / bank sentence with a blank -->
    <div v-else class="dialogue-q__sentence" lang="jbo">
      <span v-if="question.before">{{ question.before }}</span>
      <input
        v-if="question.kind === 'fill'"
        ref="inputRef"
        v-model="typed"
        class="dialogue-q__input"
        :disabled="answered || busy"
        :aria-label="t('dialogue.typeAnswer')"
        autocapitalize="off"
        autocomplete="off"
        spellcheck="false"
        @keydown.enter.prevent="submit"
      />
      <span v-else class="dialogue-q__slot">
        <button
          v-for="chip in selectedChips"
          :key="chip.id"
          type="button"
          class="dialogue-q__chip"
          :disabled="answered || busy"
          @click="unpick(chip.id)"
        >
          {{ chip.text }}
        </button>
        <span v-if="!selectedChips.length" class="dialogue-q__slot-empty">&nbsp;</span>
      </span>
      <span v-if="question.after">{{ question.after }}</span>
    </div>
    <div v-if="question.kind === 'bank'" class="dialogue-q__bank">
      <button
        v-for="chip in chips"
        :key="chip.id"
        type="button"
        class="dialogue-q__chip"
        :class="{ 'dialogue-q__chip--used': selectedIds.includes(chip.id) }"
        :disabled="answered || busy || selectedIds.includes(chip.id)"
        lang="jbo"
        @click="selectedIds.push(chip.id)"
      >
        {{ chip.text }}
      </button>
    </div>

    <div
      v-if="result"
      class="dialogue-q__result"
      :class="result.correct ? 'dialogue-q__result--ok' : 'dialogue-q__result--bad'"
      role="status"
    >
      <strong>{{
        result.correct ? t('flashcardStudy.nicelyDone') : t('dialogue.notQuite')
      }}</strong>
      <span v-if="!result.correct && result.expected">
        {{ t('flashcardStudy.correctSolution') }} <span lang="jbo">{{ result.expected }}</span>
      </span>
    </div>

    <div class="dialogue-q__actions">
      <button
        v-if="!answered"
        type="button"
        class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
        :disabled="!currentAnswer.trim() || busy"
        @click="submit"
      >
        {{ t('dialogue.check') }}
      </button>
      <button
        v-else
        type="button"
        class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
        :disabled="busy"
        @click="emit('next')"
      >
        {{ t('flashcardStudy.continue') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { DialogueChip, DialogueGrade, DialogueQuestion } from './types'

const props = defineProps<{
  question: DialogueQuestion
  total: number
  chips: DialogueChip[]
  result: DialogueGrade | null
  busy: boolean
}>()
const emit = defineEmits<{ submit: [answer: string]; next: [] }>()
const { t } = useI18n()

const typed = ref('')
const picked = ref('')
const selectedIds = ref<string[]>([])
const inputRef = ref<HTMLInputElement | null>(null)
const answered = computed(() => props.result != null)

const selectedChips = computed(() =>
  selectedIds.value
    .map((id) => props.chips.find((c) => c.id === id))
    .filter((c): c is DialogueChip => !!c)
)

const currentAnswer = computed(() => {
  if (props.question.kind === 'choice') return picked.value
  if (props.question.kind === 'bank') return selectedChips.value.map((c) => c.text).join(' ')
  return typed.value
})

function unpick(id: string) {
  selectedIds.value = selectedIds.value.filter((x) => x !== id)
}

function optionClass(text: string) {
  if (!answered.value) return { 'dialogue-q__option--picked': picked.value === text }
  const opt = props.question.options.find((o) => o.text === text)
  return {
    'dialogue-q__option--ok': opt?.correct,
    'dialogue-q__option--bad': picked.value === text && !opt?.correct,
  }
}

function submit() {
  if (answered.value || props.busy || !currentAnswer.value.trim()) return
  emit('submit', currentAnswer.value)
}

onMounted(async () => {
  await nextTick()
  inputRef.value?.focus()
})
</script>

<style scoped>
.dialogue-q {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
  border: 2px solid #e5e7eb;
  border-radius: 1rem;
  background: #fff;
}
.dialogue-q__label {
  font-size: 0.75rem;
  font-weight: 700;
  color: #6b7280;
  text-transform: uppercase;
}
.dialogue-q__prompt {
  font-weight: 600;
}
.dialogue-q__options {
  display: grid;
  gap: 0.5rem;
}
.dialogue-q__option {
  text-align: start;
  padding: 0.6rem 0.8rem;
  border: 2px solid #e5e7eb;
  border-bottom-width: 4px;
  border-radius: 0.75rem;
  background: #fff;
}
.dialogue-q__option--picked {
  border-color: #84d8ff;
  background: #ddf4ff;
}
.dialogue-q__option--ok {
  border-color: #58cc02;
  background: #d7ffb8;
}
.dialogue-q__option--bad {
  border-color: #ff4b4b;
  background: #ffdfe0;
}
.dialogue-q__sentence {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.05rem;
}
.dialogue-q__input {
  min-width: 6rem;
  flex: 1 1 8rem;
  padding: 0.3rem 0.5rem;
  border: 0;
  border-bottom: 2px solid #9ca3af;
  background: transparent;
  outline: none;
}
.dialogue-q__input:focus {
  border-color: #1cb0f6;
}
.dialogue-q__slot {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  min-width: 5rem;
  min-height: 2.2rem;
  padding: 0.15rem;
  border-bottom: 2px solid #9ca3af;
}
.dialogue-q__bank {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}
.dialogue-q__chip {
  padding: 0.3rem 0.65rem;
  border: 2px solid #e5e7eb;
  border-bottom-width: 4px;
  border-radius: 0.75rem;
  background: #fff;
}
.dialogue-q__chip--used {
  color: transparent;
  background: #e5e7eb;
}
.dialogue-q__result {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.6rem 0.8rem;
  border-radius: 0.75rem;
}
.dialogue-q__result--ok {
  background: #d7ffb8;
  color: #3f7a0b;
}
.dialogue-q__result--bad {
  background: #ffdfe0;
  color: #b42323;
}
</style>
