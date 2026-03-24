<template>

  <div class="lingo-layout flex min-h-screen flex-col bg-slate-50">
     <LingoMobileHeader />
    <div class="flex flex-1 flex-col pt-14 lg:pt-0">
       <LingoSidebar class="hidden lg:block" />
      <main class="lingo-main min-h-full flex-1 pt-6">

        <div class="mx-auto max-w-[1056px] px-4 lg:px-6"> <slot /> </div>

      </main>

    </div>

  </div>

</template>

<script setup lang="ts">
import LingoMobileHeader from './LingoMobileHeader.vue'
import LingoSidebar from './LingoSidebar.vue'
</script>

