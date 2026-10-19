<script setup lang="ts">
/**
 * Legal fine print. Shown at the end of page content when the route does not set `hideFooter`.
 */
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const visible = computed(() => !route.meta.hideFooter)
</script>

<template>
  <aside v-if="visible" class="public-domain-notice" role="note">
    <p class="public-domain-notice__text">
      {{ $t('footer.publicDomainNotice') }}
    </p>
  </aside>
</template>
