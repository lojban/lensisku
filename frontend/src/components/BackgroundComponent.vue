<template>

  <div :id="id" :class="['background-root', classes]">
    <div
      class="bg-layer"
      :class="{ 'bg-layer--top': activeLayer === 0 }"
      :style="layer0Style"
    />
    <div
      class="bg-layer"
      :class="{ 'bg-layer--top': activeLayer === 1 }"
      :style="layer1Style"
    />
  </div>

</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

const TRANSITION_S = 2.9
const transitionDurationCss = `${TRANSITION_S}s`

const props = defineProps({
  id: {
    type: String,
    default: 'background-container',
  },
  classes: {
    type: String,
    default: '',
  },
})

const backgroundUrls = [
  '/assets/backgrounds/1897_Schischkin_Im_Park_anagoria.webp',
  '/assets/backgrounds/Ivan_Ivanovich_Shishkin_-_Oaks,_1865.webp',
  '/assets/backgrounds/Ivan_Shishkin_-_Рожь_-_Google_Art_Project.webp',
  '/assets/backgrounds/Looking_Down_Yosemite-Valley.webp',
  '/assets/backgrounds/Swiss_Landscape_(Shishkin).webp',
  '/assets/backgrounds/The_Great_Wave_off_Kanagawa.webp',
  '/assets/backgrounds/Thomas_Cole_-_View_from_Mount_Holyoke,_Northampton,_Massachusetts,_after_a_Thunderstorm-The_Oxbow.webp',
  '/assets/backgrounds/Utro_v_sosnovom_lesu.webp',
  '/assets/backgrounds/View_near_Düsseldorf_(Shishkin).webp',
  '/assets/backgrounds/В_парке_(Шишкин).webp',
  '/assets/backgrounds/Пруд_в_старом_парке_(Шишкин).webp',
  '/assets/backgrounds/Скалистый_берег_(Шишкин).webp',
  '/assets/backgrounds/У_берегов_Финского_залива_(Шишкин).webp',
]

const layer0Url = ref('')
const layer1Url = ref('')
const activeLayer = ref<0 | 1>(0)
const isInitial = ref(true)

const layer0Style = computed(() => ({
  backgroundImage: layer0Url.value ? `url(${layer0Url.value})` : 'none',
  opacity: activeLayer.value === 0 ? 1 : 0,
}))

const layer1Style = computed(() => ({
  backgroundImage: layer1Url.value ? `url(${layer1Url.value})` : 'none',
  opacity: activeLayer.value === 1 ? 1 : 0,
}))

let timeoutId: ReturnType<typeof setTimeout> | undefined

function preload(url: string): Promise<boolean> {
  return new Promise((resolve) => {
    const img = new Image()
    img.onload = () => resolve(true)
    img.onerror = () => resolve(false)
    img.src = url
  })
}

onMounted(() => {
  const setBackground = async () => {
    const index = Math.floor(Math.random() * backgroundUrls.length)
    const url = backgroundUrls[index]

    const loaded = await preload(url)
    if (!loaded) {
      timeoutId = setTimeout(setBackground, 30000)
      return
    }

    if (isInitial.value) {
      layer0Url.value = url
      activeLayer.value = 0
      isInitial.value = false
    } else {
      const incoming: 0 | 1 = activeLayer.value === 0 ? 1 : 0
      if (incoming === 0) {
        layer0Url.value = url
      } else {
        layer1Url.value = url
      }
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
      })
      activeLayer.value = incoming
    }

    timeoutId = setTimeout(setBackground, 30000)
  }

  setBackground()
})

onUnmounted(() => {
  if (timeoutId !== undefined) {
    clearTimeout(timeoutId)
  }
})
</script>

<style scoped>
.background-root {
  position: relative;
  overflow: hidden;
}

.bg-layer {
  position: absolute;
  inset: 0;
  z-index: 1;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  pointer-events: none;
  transition: opacity v-bind(transitionDurationCss) ease-in-out;
}

.bg-layer--top {
  z-index: 2;
}
</style>
