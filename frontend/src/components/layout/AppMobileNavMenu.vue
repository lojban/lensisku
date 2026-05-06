<template>
  <div
    v-show="show"
    class="fixed sm:hidden top-14 left-0 right-0 bg-white shadow-md py-2 space-y-1 z-50"
  >
    <NavLink to="/collections" class="mobile-nav-row" @click="$emit('close')">
      <GraduationCap class="h-5 w-5" /> {{ t('nav.courses') }}
    </NavLink>
    <NavLink to="/recent" class="mobile-nav-row" @click="$emit('close')">
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
    <NavLink to="/languages" class="mobile-nav-row" @click="$emit('close')">
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
      <Download class="h-5 w-5" /> {{ t('nav.cachedExports') }}
    </NavLink>
    <NavLink
      v-if="auth.state.isLoggedIn"
      to="/export"
      class="mobile-nav-row"
      @click="$emit('close')"
    >
      <Upload class="h-5 w-5" /> {{ t('nav.export') }}
    </NavLink>
    <NavLink
      v-if="auth.state.isLoggedIn && auth.state.authorities?.includes('bulk_import')"
      to="/bulk-import"
      class="mobile-nav-row"
      @click="$emit('close')"
    >
      <Download class="h-5 w-5" /> {{ t('nav.bulkImport') }}
    </NavLink>
    <div v-if="auth.state.isLoggedIn" class="my-1 border-t border-gray-200" />
    <button
      v-if="auth.state.isLoggedIn"
      type="button"
      class="mobile-nav-row--emphasis"
      @click="$emit('logout')"
    >
      <LogOut class="h-5 w-5" /> {{ t('nav.logout') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import {
  Users,
  Globe,
  Download,
  Upload,
  LogOut,
  Clock4,
  GraduationCap,
  Bot,
  Share2,
} from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import NavLink from '@/components/NavLink.vue'
import { useAuth } from '@/composables/useAuth'

defineProps({
  show: { type: Boolean, default: false },
})

defineEmits<{ close: []; logout: [] }>()

const { t } = useI18n()
const auth = useAuth()
</script>
