<template>

  <div class="flex items-center space-x-3">
     <a
      v-for="link in links"
      :key="link.name"
      :href="link.link"
      target="_blank"
      rel="noopener noreferrer"
      :class="buttons ? 'text-gray-400 hover:text-blue-400 transition-colors' : 'btn-aqua-white'"
      > <img
        :src="`/assets/icons/${link.name}.svg`"
        :alt="link.name"
        class="icon p-1 h-6 w-6 hover:opacity-80"
      /> </a
    >
  </div>

</template>

<script setup lang="ts">
defineProps({
  buttons: {
    type: Boolean,
    default: false,
  },
})

const links: { name: string; link: string }[] = [
  {
    name: 'telegram',
    link: 'https://t.me/lojban',
  },
  {
    name: 'discord',
    link: 'https://discord.gg/4KhzRzpmVr',
  },
  {
    name: 'reddit',
    link: 'https://reddit.com/r/lojban',
  },
]
</script>

