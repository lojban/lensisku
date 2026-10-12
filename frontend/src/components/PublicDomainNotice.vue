<script setup lang="ts">
/**
 * Legal fine print for public-domain content.
 * Owned by App.vue — `hideFooter` / `fullHeight` / optional `publicDomainHost` in the router.
 */
withDefaults(
  defineProps<{
    /** Sit at the bottom of a flex scroll column when content is shorter than the viewport. */
    dock?: boolean
  }>(),
  { dock: false }
)
</script>

<template>
  <aside class="public-domain-notice" :class="{ 'public-domain-notice--dock': dock }" role="note">
    <p class="public-domain-notice__text">
      {{ $t('footer.publicDomainNotice') }}
    </p>
  </aside>
</template>
