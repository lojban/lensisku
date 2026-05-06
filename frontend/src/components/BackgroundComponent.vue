<template>
  <div :id="id" :class="['background-root', classes]">
    <div class="bg-layer" :class="{ 'bg-layer--top': activeLayer === 0 }" :style="layer0Style" />

    <div class="bg-layer" :class="{ 'bg-layer--top': activeLayer === 1 }" :style="layer1Style" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

const TRANSITION_S = 2.9
const transitionDurationCss = `${TRANSITION_S}s`

defineProps({
  id: {
    type: String,
    default: 'background-container',
  },
  classes: {
    type: String,
    default: '',
  },
})

// Colloquial glosses (corpus-style phrasing; valsi checked via semantic index to sampu vlaste):
// ricfoi korbi cadzu dargu jvinu viska zgana — forest-nymph / dwe / Terry / Thlunrana / Tlön patterns
const backgroundUrls = [
  // en: A Walk in the Forest (1869) — lu le nu cadzu bu'u lo ricfoi li'u  (cadzu, ricfoi)
  '/assets/backgrounds/a-walk-in-the-forest-1869.webp',
  // en: At the Edge of a Birch Grove (1871) — lu ne'a le korbi be le ricfoi be lo bi'orka li'u  (korbi, ricfoi, bi'orka)
  '/assets/backgrounds/at-the-edge-of-a-birch-grove-1871.webp',
  // en: Autumn (1892) — lu pa critu li'u  (critu; year omitted in sumti, kept in filename)
  '/assets/backgrounds/autumn-1892.webp',
  // en: Birch Grove (1896) — lu le ricfoi be lo bi'orka li'u  (ricfoi, bi'orka)
  '/assets/backgrounds/birch-grove-1896.webp',
  // en: Countess Mordvinov's Forest (1891) — lu le ricfoi po la .mordvinov. voi noltroni'u li'u  (ricfoi, noltroni'u)
  '/assets/backgrounds/countess-mordvinovs-forest-1891.webp',
  // en: Forest Glade (1897) — lu le foldi ne'i le ricfoi li'u  (foldi, ricfoi; glade ≈ open foldi in woods)
  '/assets/backgrounds/forest-glade-1897.webp',
  // en: Forest road (1897) — lu le dargu ne'i le ricfoi li'u  (dargu, ricfoi)
  '/assets/backgrounds/forest-road-1897.webp',
  // en: In the Wild North (1891) — lu bu'u le berti tumla poi cilce li'u  (berti, tumla, cilce)
  '/assets/backgrounds/in-the-wild-north-1891.webp',
  // en: Mast Tree Grove (1898) — lu le ricfoi be lo ckunu noi lo mudri cu se pilno zo'e fi lo nu zbasu lo agmasto li'u  (ricfoi, ckunu, mudri, se pilno, nu zbasu; agmasto fu'ivla)
  '/assets/backgrounds/mast-tree-grove-1898.webp',
  // en: Misty Morning (1897) — lu le cerni poi se bumru li'u  (cerni, bumru)
  '/assets/backgrounds/misty-morning-1897.webp',
  // en: Morning in a Pine Forest (1889) — lu le cerni ne'i le ricfoi be lo ckunu li'u  (cerni, ricfoi, ckunu)
  '/assets/backgrounds/morning-in-a-pine-forest-1889.webp',
  // en: Oak Grove (1887) — lu le ricfoi be lo cindu li'u  (ricfoi, cindu)
  '/assets/backgrounds/oak-grove-1887.webp',
  // en: Oak on the shore of the Gulf of Finland (1857) — lu lo cindu noi zvati le xaskoi be le xamsi po la .su,omi. li'u  (cindu, xaskoi, xamsi)
  '/assets/backgrounds/oak-on-the-shore-of-the-gulf-of-finland-1857.webp',
  // en: Pine Forest (1872) — lu le ricfoi be lo ckunu li'u  (ricfoi, ckunu)
  '/assets/backgrounds/pine-forest-1872.webp',
  // en: Pond in an Old Park (1897) — lu le lalxu ne'i le tolci'o purdi li'u  (lalxu, tolci'o, purdi)
  '/assets/backgrounds/pond-in-an-old-park-1897.webp',
  // en: Road in the Pine Forest (1885) — lu le dargu ne'i le ricfoi be lo ckunu li'u  (dargu, ricfoi, ckunu; cf. kelda koe pailtegaxo in forest-nymph corpus)
  '/assets/backgrounds/road-in-the-pine-forest-1885.webp',
  // en: Rye (1878) — lu lo foldi be lo mraji li'u  (foldi, mraji)
  '/assets/backgrounds/rye-1878.webp',
  // en: The Edge of the Forest (1884) — lu le korbi be le ricfoi li'u  (korbi, ricfoi)
  '/assets/backgrounds/the-edge-of-the-forest-1884.webp',
  // en: The Sunlit Pines (1886) — lu lei ckunu poi se gusni fi le solri li'u  (ckunu, gusni, solri)
  '/assets/backgrounds/the-sunlit-pines-1886.webp',
  // en: View on the Island of Valaam (1858) — lu le jvinu be la .vala,ams. daplu li'u  (jvinu, daplu; cmene approximate)
  '/assets/backgrounds/view-on-the-island-of-valaam-1858.webp',
  // en: Looking Down Yosemite Valley — lu le jvinu dizlo be le ma'arbi'i be me'e zoi gy.Yosemite.gy. li'u  (jvinu, dizlo, ma'arbi'i)
  '/assets/backgrounds/Looking_Down_Yosemite-Valley.webp',
  // en: The Great Wave off Kanagawa — lu pa banli boxna be fi le xamsi ni'a la .kanagavas. li'u  (boxna, xamsi; ni'a = below, off the coast of)
  '/assets/backgrounds/The_Great_Wave_off_Kanagawa.webp',
  // en: The Oxbow (Cole) — lu le jvinu be fi la .maunt.holiok. ba le lidvilti'a li'u  (jvinu, lidvilti'a = thunderstorm)
  '/assets/backgrounds/Thomas_Cole_-_View_from_Mount_Holyoke,_Northampton,_Massachusetts,_after_a_Thunderstorm-The_Oxbow.webp',
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
