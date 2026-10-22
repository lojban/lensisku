<script setup lang="ts">
/**
 * Scroll column for `meta.fullHeight` feed pages. The public-domain notice
 * is the last child; `hideFooter` hides it.
 */
import PublicDomainNotice from '@/components/PublicDomainNotice.vue'
</script>

<template>
  <div class="feed-page__body">
    <div class="feed-page__column">
      <slot />
      <PublicDomainNotice />
    </div>
  </div>
</template>
