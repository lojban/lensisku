<template>
  <div
    :id="id"
    :class="classes"
  />
</template>

<script setup>
import { onMounted } from 'vue'

const props = defineProps({
  id: {
    type: String,
    default: 'background-container'
  },
  classes: {
    type: String,
    default: ''
  }
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

onMounted(() => {
  const bgDiv = document.getElementById(props.id)
  let timeoutId

  const setBackground = async () => {
    const index = Math.floor(Math.random() * backgroundUrls.length)
    const url = backgroundUrls[index]

    const loaded = await new Promise((resolve) => {
      const img = new Image()
      img.onload = () => resolve(true)
      img.onerror = () => resolve(false)
      img.src = url
    })

    if (loaded) {
      bgDiv.style.backgroundImage = `url(${url})`
    }

    // Schedule next background change after 30 seconds
    timeoutId = setTimeout(setBackground, 30000)
  }

  // Initial call
  setBackground()

  // Cleanup
  return () => clearTimeout(timeoutId)
})
</script>

<style scoped>

</style>
