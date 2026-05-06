<template>
  <div
    class="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900 text-white flex items-center justify-center relative"
  >
    <audio ref="audioPlayer">
      <source src="/assets/music/rikroli.mp3" type="audio/mpeg" />
    </audio>
    <div class="w-full bg-gray-800 bg-opacity-80 backdrop-blur-md shadow-2xl md:p-12">
      <button
        class="z-20 mx-auto mb-8 flex items-center gap-2 rounded-full bg-purple-600 px-6 py-3 font-bold text-white shadow-lg transition-[filter,box-shadow] duration-200 hover:bg-purple-700 hover:shadow-xl hover:brightness-110"
        @click="toggleAudio"
      >
        <Pause v-if="isPlaying" class="h-6 w-6" /> <Play v-else class="h-6 w-6" />
        <span>{{ isPlaying ? 'Pause' : 'Play Lojban Demo' }}</span>
      </button>
      <div
        v-if="showLyrics"
        class="mt-8 p-6 bg-gray-700 bg-opacity-70 rounded-lg shadow-inner max-w-2xl mx-auto"
      >
        <h3 class="text-xl font-semibold text-purple-300 mb-4 text-center">do e mi prami pu</h3>

        <pre
          class="text-gray-200 whitespace-pre-wrap text-center font-mono text-sm leading-relaxed"
          >{{ lojbanLyrics }}</pre
        >
      </div>

      <h1
        class="text-4xl md:text-5xl font-serif font-bold text-center mb-6 text-transparent bg-clip-text bg-gradient-to-r from-amber-400 via-red-500 to-purple-600"
      >
        Lojban 2025: Now With Mandatory Credit System!
      </h1>

      <p class="text-lg text-gray-300 mb-8 text-center italic">
        For too long, Lojban has been the playground of intellectual elitists. Those "logical
        language" nerds with their "reasoned arguments" and "community consensus"? Gone.
      </p>

      <h2 class="text-2xl font-semibold text-purple-400 mb-6 text-center">
        Introducing Lojban Premium™ - where usage of the language deducts from your credit balance!
      </h2>

      <ul
        class="space-y-4 mb-10 text-gray-200 list-disc list-inside pl-4 md:pl-8 text-base md:text-lg"
      >
        <li>
          <span class="font-medium text-amber-400">Want to take a course?</span> 499 credits basic
          tier (vocabulary sold separately)
        </li>

        <li>
          <span class="font-medium text-amber-400">Want to propose a new word?</span> That&#39;ll be
          5 credits per letter.
        </li>

        <li>
          <span class="font-medium text-amber-400">Need a grammar clarification?</span> Premium
          Support package starts at 19 credits/month.
        </li>

        <li>
          <span class="font-medium text-amber-400">Translation services?</span> Basic tier: 0.1
          credits per word (minimum 1000 words).
        </li>

        <li>
          <span class="font-medium text-amber-400">Just want to say "coi" (hello)?</span>
          Microtransaction: 1 credit per greeting.
        </li>
      </ul>

      <p class="text-lg text-gray-300 mb-8 text-center font-semibold">
        Bonus feature: Our revolutionary "Pay-Per-Think" system ensures you&#39;re only charged when
        you <em>actually</em> understand what you&#39;re saying!
      </p>

      <p class="text-xs text-gray-500 text-center mt-10">
        *Terms and conditions apply. All credits non-refundable. April 1st special: First 100
        complaints processed for just 99 credits each!*
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Play, Pause } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
useSeoHead({ title: t('aprilFools.pageTitle') })

const audioPlayer = ref(null)
const isPlaying = ref(false)
const showLyrics = ref(false)

const lojbanLyrics = `
i do e mi prami pu
i mi'o djuno fi le javni
i se'o traji prami fa mi au
i fa no drata nanmu ka'e gasnu
i sa'u au mi fi do skicu
i au fa do co'a jimpe

i ai ze'e sidju do i ai no roi cliva do
i ai no roi jai gau badri fai do
i ai noroi to'e rinsa do i ai no roi jai gau tcica do
i ai no roi jai gau klaku fai do

i mi pu ze'u slabu do
i do pu cinmo gi'e mutce cumla
i ku'i mi'o djuno fi le fasnu iu
i melbi javni ju'o dai i ai ci'erkei
i fa ko ganai kucli tu'a le se cinmo
gi na ku rivbi lo ka jimpe fi mi
`.trim()
const toggleAudio = () => {
  if (!audioPlayer.value) return

  if (isPlaying.value) {
    audioPlayer.value.pause()
    isPlaying.value = false
  } else {
    audioPlayer.value
      .play()
      .then(() => {
        isPlaying.value = true
        showLyrics.value = true // Show lyrics when audio plays
      })
      .catch((error) => {
        console.error('Audio play failed:', error)
      })
  }
}

onMounted(() => {
  // Initialize audio player
  if (audioPlayer.value) {
    audioPlayer.value.addEventListener('ended', () => {
      isPlaying.value = false
    })
  }
})
</script>
