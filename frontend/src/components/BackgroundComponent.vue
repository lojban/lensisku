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
  'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e7/%D0%A3_%D0%B1%D0%B5%D1%80%D0%B5%D0%B3%D0%BE%D0%B2_%D0%A4%D0%B8%D0%BD%D1%81%D0%BA%D0%BE%D0%B3%D0%BE_%D0%B7%D0%B0%D0%BB%D0%B8%D0%B2%D0%B0_%28%D0%A3%D0%B4%D1%80%D0%B8%D0%B0%D1%81_%D0%B1%D0%BB%D0%B8%D0%B7_%D0%9D%D0%B0%D1%80%D0%B2%D1%8B%29_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg/665px-%D0%A3_%D0%B1%D0%B5%D1%80%D0%B5%D0%B3%D0%BE%D0%B2_%D0%A4%D0%B8%D0%BD%D1%81%D0%BA%D0%BE%D0%B3%D0%BE_%D0%B7%D0%B0%D0%BB%D0%B8%D0%B2%D0%B0_%28%D0%A3%D0%B4%D1%80%D0%B8%D0%B0%D1%81_%D0%B1%D0%BB%D0%B8%D0%B7_%D0%9D%D0%B0%D1%80%D0%B2%D1%8B%29_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg?20190728095332',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/e/eb/Ivan_Shishkin_-_%D0%A0%D0%BE%D0%B6%D1%8C_-_Google_Art_Project.jpg/800px-Ivan_Shishkin_-_%D0%A0%D0%BE%D0%B6%D1%8C_-_Google_Art_Project.jpg?20110220152115',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/0/0a/The_Great_Wave_off_Kanagawa.jpg/800px-The_Great_Wave_off_Kanagawa.jpg?20070829004226',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/1/16/Swiss_Landscape_%28Shishkin%29.jpg/800px-Swiss_Landscape_%28Shishkin%29.jpg?20170129134644',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/2/20/Utro_v_sosnovom_lesu.jpg/800px-Utro_v_sosnovom_lesu.jpg?20160515171643',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/2/2e/View_from_Mount_Holyoke%2C_Northampton%2C_Massachusetts%2C_after_a_Thunderstorm%E2%80%94The_Oxbow_MET_DP-12550-007.jpg/1280px-View_from_Mount_Holyoke%2C_Northampton%2C_Massachusetts%2C_after_a_Thunderstorm%E2%80%94The_Oxbow_MET_DP-12550-007.jpg',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/4/45/Dub_Shishkin.jpg/512px-Dub_Shishkin.jpg?20170205025941',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/4/48/%D0%92_%D0%BF%D0%B0%D1%80%D0%BA%D0%B5_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg/800px-%D0%92_%D0%BF%D0%B0%D1%80%D0%BA%D0%B5_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg?20180530070024',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/4/4e/1897_Schischkin_Im_Park_anagoria.JPG/800px-1897_Schischkin_Im_Park_anagoria.JPG?20120127203716',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/7/75/%D0%9F%D1%80%D1%83%D0%B4_%D0%B2_%D1%81%D1%82%D0%B0%D1%80%D0%BE%D0%BC_%D0%BF%D0%B0%D1%80%D0%BA%D0%B5_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg/800px-%D0%9F%D1%80%D1%83%D0%B4_%D0%B2_%D1%81%D1%82%D0%B0%D1%80%D0%BE%D0%BC_%D0%BF%D0%B0%D1%80%D0%BA%D0%B5_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg?20171124050104',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/7/78/%D0%A1%D0%BA%D0%B0%D0%BB%D0%B8%D1%81%D1%82%D1%8B%D0%B9_%D0%B1%D0%B5%D1%80%D0%B5%D0%B3_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg/800px-%D0%A1%D0%BA%D0%B0%D0%BB%D0%B8%D1%81%D1%82%D1%8B%D0%B9_%D0%B1%D0%B5%D1%80%D0%B5%D0%B3_%28%D0%A8%D0%B8%D1%88%D0%BA%D0%B8%D0%BD%29.jpg?20181117172838',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/8/8c/Looking_Down_Yosemite-Valley.jpg/800px-Looking_Down_Yosemite-Valley.jpg',
  'https://upload.wikimedia.org/wikipedia/commons/thumb/f/fa/View_near_D%C3%BCsseldorf_%28Shishkin%29.jpg/800px-View_near_D%C3%BCsseldorf_%28Shishkin%29.jpg?20170204165503',
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
