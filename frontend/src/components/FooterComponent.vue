<template>
   <!-- Popup for chat controls --> <ModalComponent
    :show="showPopup"
    :title="t('footer.chatSettings')"
    @close="showPopup = false"
    > <SocialLinks class="mb-4" /> <!-- Toggle for marquee -->
    <div class="flex items-center justify-between mb-4">
       <span class="text-gray-700">{{ t('footer.marqueeToggle') }}</span
      > <button
        class="relative inline-flex flex-shrink-0 h-6 w-11 border-2 border-transparent rounded-full cursor-pointer transition-colors ease-in-out duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        :class="marqueeEnabled ? 'bg-blue-500' : 'bg-gray-200'"
        @click="toggleMarquee"
      >
         <span
          aria-hidden="true"
          class="inline-block h-5 w-5 rounded-full bg-white shadow transform transition ease-in-out duration-200"
          :class="marqueeEnabled ? 'translate-x-5' : 'translate-x-0'"
        /> </button
      >
    </div>
     <!-- Last 10 messages -->
    <div class="space-y-2">

      <h4 class="text-gray-700 font-medium mb-2"> {{ t('footer.recentMessages') }} </h4>

      <div v-if="messageStack.length > 0" class="max-h-[60vh] overflow-y-auto pr-2">

        <div
          v-for="(msg, index) in messageStack.slice()"
          :key="index"
          class="p-2 bg-gray-50 rounded break-words whitespace-normal"
        >
           <span class="font-medium text-blue-600">{{ msg.w }}:</span> <span
            class="text-gray-700 ml-1"
            >{{ msg.d }}</span
          >
        </div>

      </div>

      <div v-else class="text-gray-500"> {{ t('footer.noRecentMessages') }} </div>

    </div>
     </ModalComponent
  >
  <footer
    class="fixed bottom-0 left-0 right-0 h-6 bg-white border-t border-gray-200 z-40 shadow-sm"
  >

    <div class="max-w-6xl w-full mx-auto h-full flex items-center justify-between pl-4">

      <div class="flex items-center w-full">
         <SocialLinks class="mr-2" :buttons="true" />
        <div class="relative group mr-4 hidden md:flex">
           <select
            :value="locale"
            class="input-field appearance-none !h-5 !py-0 !pr-8 !text-xs"
            @change="switchLanguage"
          >

            <option v-for="loc in availableLocales" :key="`locale-${loc}`" :value="loc">
               {{ loc.toUpperCase() }}
            </option>
             </select
          > <ChevronDown
            class="z-[80] h-4 w-4 text-gray-600 absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none"
          />
        </div>

        <div class="overflow-hidden flex-1 min-w-0 relative" @click="showPopup = true">

          <div
            class="marquee whitespace-nowrap"
            :class="{ '!hidden': !marqueeEnabled || messageStack.length === 0 }"
          >
             <span
              id="velsku_sebenji"
              class="text-blue-500 italic hover:opacity-80 transition-colors text-sm"
            />
          </div>

          <div
            class="absolute inset-y-0 left-0 w-10 bg-gradient-to-r from-white to-transparent pointer-events-none"
          />

          <div
            class="absolute inset-y-0 right-0 w-10 bg-gradient-to-l from-white to-transparent pointer-events-none"
          />

          <div class="whitespace-nowrap" :class="{ hidden: marqueeEnabled }">
             <span
              class="cursor-pointer text-blue-500 italic hover:opacity-80 transition-colors text-sm"
              >{{ $t('footer.liveChat') }}</span
            >
          </div>

        </div>

      </div>

    </div>

  </footer>

</template>

<script setup lang="ts">
import { io } from 'socket.io-client'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { ChevronDown } from 'lucide-vue-next'

import ModalComponent from './ModalComponent.vue'
import SocialLinks from './SocialLinks.vue'

const { locale, availableLocales, t } = useI18n()
const router = useRouter()
const route = useRoute()

const showPopup = ref(false)
const marqueeEnabled = ref(
  typeof window === 'undefined' ? false : localStorage.getItem('marqueeEnabled') !== 'false'
)
const socket1Chat_connected = ref(false)
const messageStack = ref([])

const toggleMarquee = () => {
  if (typeof window === 'undefined') return

  marqueeEnabled.value = !marqueeEnabled.value
  localStorage.setItem('marqueeEnabled', marqueeEnabled.value ? 'true' : 'false')
}

const switchLanguage = (event) => {
  if (typeof window === 'undefined') return

  const newLocale = event.target.value
  if (availableLocales.includes(newLocale)) {
    locale.value = newLocale
    localStorage.setItem('selectedLocale', newLocale)

    const currentPath = route.path
    const pathSegments = currentPath.split('/').filter((segment) => segment !== '')

    if (pathSegments.length > 0 && availableLocales.includes(pathSegments[0])) {
      pathSegments[0] = newLocale
    } else {
      pathSegments.unshift(newLocale)
    }

    const newPath = '/' + pathSegments.join('/')
    router.push({ path: newPath, query: route.query })
  }
}

const updateChatMessage = (messages) => {
  const velsku = document.getElementById('velsku_sebenji')
  if (velsku) {
    const sanitizedMessages = messages.map((m) => ({
      ...m,
      d: m.d
        .replace(/<[^>]*>?/gm, '')
        .replace(/\s+/g, ' ')
        .trim(),
    }))

    messageStack.value.push(...sanitizedMessages)
    messageStack.value = messageStack.value.slice(-10)

    const formattedMessages = messageStack.value.map((m) => `${m.w}: ${m.d}`).join('   •   ')
    velsku.innerHTML = formattedMessages
  }
}

onMounted(() => {
  const socket1Chat = io('wss://jbotcan.org:9091', {
    transports: ['polling', 'websocket'],
  })

  socket1Chat.on('connect', () => {
    socket1Chat_connected.value = true
  })

  socket1Chat.on('connect_error', () => {
    console.error('1chat connection error')
  })

  socket1Chat.on('sentFrom', (data) => {
    if (!socket1Chat_connected.value) return
    const i = data.data
    updateChatMessage([
      {
        d: i.chunk,
        s: i.channelId,
        w: i.author,
      },
    ])
  })

  socket1Chat.on('history', (data) => {
    if (!socket1Chat_connected.value) return
    updateChatMessage(
      data.slice(-10).map((m) => ({
        d: m.chunk,
        s: m.channelId,
        w: m.author,
      }))
    )
  })
})
</script>

<style scoped>
.marquee {
  display: inline-block;
  cursor: pointer;
  animation: marquee 200s linear infinite;
}

@keyframes marquee {
  0% {
    transform: translateX(0);
  }

  100% {
    transform: translateX(-100%);
  }
}
</style>

