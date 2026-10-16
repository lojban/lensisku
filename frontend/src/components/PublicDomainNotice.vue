<script setup lang="ts">
/**
 * Legal fine print for public-domain content.
 * Visibility/docking come from route meta (`hideFooter`, `fullHeight`) — no Teleport.
 */
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const visible = computed(() => !route.meta.hideFooter)
/** Dock to the bottom of a flex scroll column on fullHeight routes. */
const dock = computed(() => !!route.meta.fullHeight)
</script>

<template>
  <aside
    v-if="visible"
    class="public-domain-notice"
    :class="{ 'public-domain-notice--dock': dock }"
    role="note"
  >
    <p class="public-domain-notice__text">
      {{ $t('footer.publicDomainNotice') }}
    </p>
  </aside>
</template>
