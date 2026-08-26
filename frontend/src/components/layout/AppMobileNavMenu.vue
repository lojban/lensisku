<template>
  <div
    v-show="show"
    class="fixed sm:hidden top-14 left-0 right-0 bg-white shadow-md py-2 space-y-2 z-50"
  >
    <div class="mobile-nav-grid">
      <NavLink
        to="/collections"
        class="mobile-nav-row"
        :class="{ 'mobile-nav-grid__full': !auth.state.isLoggedIn }"
        @click="$emit('close')"
      >
        <GraduationCap class="h-5 w-5" /> {{ t('nav.learn') }}
      </NavLink>
      <NavLink
        v-if="auth.state.isLoggedIn"
        to="/library"
        class="mobile-nav-row"
        @click="$emit('close')"
      >
        <Star class="h-5 w-5" /> {{ t('nav.library') }}
      </NavLink>
      <NavLink v-if="auth.state.isLoggedIn" to="/mi" class="mobile-nav-row" @click="$emit('close')">
        <BookmarkCheck class="h-5 w-5" /> {{ t('mobileNav.myActivityAndProfile') }}
      </NavLink>
      <Button
        v-if="auth.state.isLoggedIn"
        variant="plain"
        type="button"
        class="nav-link text-nav-link mobile-nav-row w-full"
        @click="$emit('logout')"
      >
        <LogOut class="h-5 w-5" /> {{ t('nav.logout') }}
      </Button>
    </div>
    <div class="my-2 border-t border-gray-200" />
    <div class="mobile-nav-grid">
      <NavLink to="/recent" class="mobile-nav-row mobile-nav-grid__full" @click="$emit('close')">
        <Clock4 class="h-5 w-5" /> {{ t('mobileNav.recentChanges') }}
      </NavLink>
      <NavLink
        v-if="auth.state.isLoggedIn"
        to="/users"
        class="mobile-nav-row"
        @click="$emit('close')"
      >
        <Users class="h-5 w-5" />
        {{ auth.state.authorities?.includes('manage_roles') ? t('nav.iamUsers') : t('nav.users') }}
      </NavLink>
      <NavLink
        to="/languages"
        class="mobile-nav-row"
        :class="{ 'mobile-nav-grid__full': !auth.state.isLoggedIn }"
        @click="$emit('close')"
      >
        <Globe class="h-5 w-5" /> {{ t('nav.languages') }}
      </NavLink>
      <NavLink to="/assistant" class="mobile-nav-row" @click="$emit('close')">
        <Bot class="h-5 w-5" /> {{ t('nav.assistant') }}
      </NavLink>
      <NavLink to="/semantic-graph" class="mobile-nav-row" @click="$emit('close')">
        <Share2 class="h-5 w-5" /> {{ t('nav.semanticGraph') }}
      </NavLink>
      <NavLink
        v-if="!auth.state.isLoggedIn"
        to="/export/cached"
        class="mobile-nav-row"
        @click="$emit('close')"
      >
        <DownloadIcon class="h-5 w-5" /> {{ t('nav.cachedExports') }}
      </NavLink>
    </div>
    <div v-if="auth.state.isLoggedIn" class="mobile-nav-grid">
      <NavLink
        to="/export"
        class="mobile-nav-row"
        :class="{ 'mobile-nav-grid__full': !canBulkImport }"
        @click="$emit('close')"
      >
        <ExportIcon class="h-5 w-5" /> {{ t('nav.export') }}
      </NavLink>
      <NavLink v-if="canBulkImport" to="/bulk-import" class="mobile-nav-row" @click="$emit('close')">
        <ImportIcon class="h-5 w-5" /> {{ t('nav.bulkImport') }}
      </NavLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, DownloadIcon, ExportIcon, ImportIcon } from '@packages/ui'
import {
  Users,
  Globe,
  LogOut,
  Clock4,
  GraduationCap,
  Star,
  BookmarkCheck,
  Bot,
  Share2,
} from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import NavLink from '@/components/NavLink.vue'
import { useAuth } from '@/composables/useAuth'

defineProps({
  show: { type: Boolean, default: false },
})

defineEmits<{ close: []; logout: [] }>()

const { t } = useI18n()
const auth = useAuth()
const canBulkImport = computed(() => Boolean(auth.state.authorities?.includes('bulk_import')))
</script>
