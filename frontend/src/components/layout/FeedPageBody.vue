<script setup lang="ts">
/**
 * Scroll column for `meta.fullHeight` feed pages. Renders the public-domain notice
 * from route meta at the end of the column (no App Teleport).
 */
import PublicDomainNotice from '@/components/PublicDomainNotice.vue'
</script>

<template>
  <div class="feed-page__body">
    <slot />
    <PublicDomainNotice />
  </div>
</template>
