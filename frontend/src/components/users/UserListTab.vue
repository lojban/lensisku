<template>

  <div class="space-y-6">
    <!-- Search and Filter Controls (compact single row) -->
    <div class="bg-white border border-gray-200 rounded-xl shadow-sm p-3 sm:p-4">

      <div class="flex flex-wrap items-center gap-3 sm:gap-4">
        <!-- Search: compact width -->
        <div class="w-full sm:w-auto sm:min-w-[220px] sm:max-w-[280px] flex-1 sm:flex-initial">
          <SearchInput :model-value="searchQuery" :is-loading="isSearching"
            :placeholder="t('userList.searchPlaceholder')" :show-search-icon="true"
            @update:model-value="$emit('update:searchQuery', $event)" @keyup.enter="$emit('updateSearch')"
            @clear="$emit('clearSearch')" />
        </div>
        <!-- Role Filter -->
        <div class="flex items-center gap-2 shrink-0">
          <label class="text-sm font-medium text-gray-700 whitespace-nowrap">{{
            t('components.userListTab.roleLabel')
          }}</label>
          <div class="relative shrink-0">
            <Dropdown>
              <template #trigger="{ open: roleMenuOpen }">
                <button
                  type="button"
                  class="input-field inline-flex h-8 w-auto max-w-[min(100vw-4rem,14rem)] items-center justify-between gap-1.5 px-3 text-sm"
                >
                  <span class="whitespace-nowrap truncate">{{ roleFilterTriggerLabel }}</span>
                  <ChevronDown
                    class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
                    :class="{ 'rotate-180': roleMenuOpen }"
                    :stroke-width="2"
                  />
                </button>
              </template>
              <button
                type="button"
                class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                @click="$emit('update:roleFilter', '')"
              >
                {{ t('components.userListTab.allRoles') }}
              </button>
              <button
                v-for="role in availableRoles"
                :key="role.name"
                type="button"
                class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                @click="$emit('update:roleFilter', role.name)"
              >
                {{ translateRole(role.name) }}
              </button>
            </Dropdown>
          </div>
        </div>
        <!-- Sort by + order (button group) -->
        <div class="flex w-auto max-w-full min-w-0 shrink-0 items-center gap-2">
          <label class="w-auto shrink-0 text-sm font-medium text-gray-700 whitespace-nowrap" for="user-list-sort-by">{{
            t('components.userListTab.sortByLabel')
          }}</label>
          <div
            class="flex w-auto shrink-0 flex-nowrap items-stretch"
            role="group"
            :aria-label="sortControlsAriaLabel"
          >
            <div
              class="relative flex min-h-8 w-auto shrink-0 flex-col justify-center min-w-0 [&>div]:w-auto"
            >
              <Dropdown>
                <template #trigger="{ open: sortByMenuOpen }">
                  <button
                    id="user-list-sort-by"
                    type="button"
                    class="input-field inline-flex h-8 w-auto max-w-full min-w-0 items-center justify-between gap-1.5 px-3 text-sm text-left rounded-r-none border-r-0"
                  >
                    <span class="whitespace-nowrap">{{ sortByTriggerLabel }}</span>
                    <ChevronDown
                      class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
                      :class="{ 'rotate-180': sortByMenuOpen }"
                      :stroke-width="2"
                    />
                  </button>
                </template>
                <button
                  type="button"
                  class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                  @click="$emit('update:sortBy', 'created_at')"
                >
                  {{ t('components.userListTab.createdAtSort') }}
                </button>
                <button
                  type="button"
                  class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                  @click="$emit('update:sortBy', 'username')"
                >
                  {{ t('components.userListTab.usernameSort') }}
                </button>
                <button
                  type="button"
                  class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                  @click="$emit('update:sortBy', 'realname')"
                >
                  {{ t('components.userListTab.realNameSort') }}
                </button>
              </Dropdown>
            </div>
            <div
              class="relative flex min-h-8 w-auto shrink-0 flex-col justify-center min-w-0 [&>div]:w-auto"
            >
              <Dropdown>
                <template #trigger="{ open: orderMenuOpen }">
                  <button
                    type="button"
                    class="input-field inline-flex h-8 w-auto max-w-full min-w-0 items-center justify-between gap-1.5 px-3 text-sm text-left rounded-l-none border-l-0"
                    :aria-label="t('components.userListTab.sortOrderLabel')"
                  >
                    <span class="whitespace-nowrap">{{ sortOrderTriggerLabel }}</span>
                    <ChevronDown
                      class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
                      :class="{ 'rotate-180': orderMenuOpen }"
                      :stroke-width="2"
                    />
                  </button>
                </template>
                <button
                  type="button"
                  class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                  @click="$emit('update:sortOrder', 'asc')"
                >
                  {{ t('components.userListTab.ascSort') }}
                </button>
                <button
                  type="button"
                  class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
                  @click="$emit('update:sortOrder', 'desc')"
                >
                  {{ t('components.userListTab.descSort') }}
                </button>
              </Dropdown>
            </div>
          </div>
        </div>

      </div>

    </div>
    <!-- User list -->
    <div class="min-h-[400px]">
      <!-- Loading state -->
      <div v-if="isLoading && userList.length === 0" class="flex flex-col items-center justify-center py-16">

        <div class="animate-spin rounded-full h-10 w-10 border-2 border-blue-500 border-t-transparent"
          aria-hidden="true" />

        <p class="mt-3 text-sm text-gray-600">{{ t('userList.loadingUsers') }}</p>

      </div>

      <div v-else class="grid gap-3 sm:gap-4">

        <div v-for="user in userList" :key="user.user_id"
          class="min-w-0 max-w-full bg-white p-4 sm:p-5 rounded-xl border border-gray-200 hover:border-blue-400/60 hover:shadow-md transition-all duration-200 cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          role="button" tabindex="0" @click="$emit('viewUser', user.username)"
          @keyup.enter="$emit('viewUser', user.username)">

          <div class="flex items-start gap-4">
            <!-- Avatar -->
            <div class="shrink-0 mt-1">

              <div v-if="user.has_profile_image"
                class="w-12 h-12 rounded-full overflow-hidden border border-gray-100 shadow-sm">
                <img :src="getProfileImage(user.username, { cached: true })" :alt="user.username"
                  class="w-full h-full object-cover" />
              </div>

              <div v-else
                class="w-12 h-12 rounded-full bg-gray-50 flex items-center justify-center text-gray-400 border border-gray-100 shadow-sm">
                <User class="h-6 w-6" />
              </div>

            </div>
            <!-- User Info -->
            <div class="min-w-0 flex-1">

              <div class="flex justify-between items-start gap-2 min-w-0">

                <div class="min-w-0 flex-1 pr-1">

                  <h3 class="text-lg font-medium text-blue-600 break-words hover:text-blue-700">
                    {{ user.username }}
                  </h3>

                  <p v-if="user.realname" class="text-gray-600 text-sm mt-0.5 break-words">
                    {{ user.realname }}
                  </p>

                </div>
                <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium shrink-0"
                  :class="getRoleClass(user.role)"> {{ translateRole(user.role) }} </span>
              </div>
              <!-- Personal description -->
              <p v-if="user.personal" class="text-gray-500 text-sm mt-2 line-clamp-2">
                {{ user.personal }}
              </p>
              <!-- Join Date -->
              <div class="flex items-center gap-1.5 mt-3 text-xs text-gray-400">
                <Calendar class="h-3.5 w-3.5" /> <span>{{
                  t('components.userListTab.joinedAt', { date: formatDate(user.created_at) })
                }}</span>
              </div>

            </div>

          </div>

        </div>

      </div>

    </div>
    <PaginationComponent v-if="total > perPage" :current-page="currentPage" :total-pages="totalPages" :total="total"
      :per-page="perPage" class="mt-6" @prev="$emit('prevPage')" @next="$emit('nextPage')" />
  </div>

</template>

<script setup lang="ts">
import { Calendar, ChevronDown, User } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { computed, type PropType } from 'vue'
import { getProfileImage } from '@/api'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SearchInput from '@/components/SearchInput.vue'
import { Dropdown } from '@packages/ui'

const { t, locale } = useI18n()

type UserRow = {
  user_id: string | number
  username: string
  realname?: string | null
  role?: string
  personal?: string | null
  has_profile_image?: boolean
  created_at?: string
}

const props = defineProps({
  userList: { type: Array as PropType<UserRow[]>, required: true },
  total: { type: Number, required: true },
  perPage: { type: Number, required: true },
  currentPage: { type: Number, required: true },
  totalPages: { type: Number, required: true },
  availableRoles: { type: Array as PropType<Array<{ name: string }>>, required: true },
  isLoading: { type: Boolean, required: true },
  isSearching: { type: Boolean, required: true },
  searchQuery: { type: String, required: true },
  roleFilter: { type: String, required: true },
  sortBy: { type: String, required: true },
  sortOrder: { type: String, required: true },
})

const roleFilterTriggerLabel = computed(() => {
  if (!props.roleFilter) return t('components.userListTab.allRoles')
  return translateRole(props.roleFilter)
})

const sortByTriggerLabel = computed(() => {
  if (props.sortBy === 'username') return t('components.userListTab.usernameSort')
  if (props.sortBy === 'realname') return t('components.userListTab.realNameSort')
  return t('components.userListTab.createdAtSort')
})

const sortOrderTriggerLabel = computed(() =>
  props.sortOrder === 'asc' ? t('components.userListTab.ascSort') : t('components.userListTab.descSort'),
)

const sortControlsAriaLabel = computed(
  () => `${t('components.userListTab.sortByLabel')} ${t('components.userListTab.sortOrderLabel')}`,
)

defineEmits([
  'update:searchQuery',
  'update:roleFilter',
  'update:sortBy',
  'update:sortOrder',
  'updateSearch',
  'clearSearch',
  'prevPage',
  'nextPage',
  'viewUser',
])

const formatDate = (dateString: string | undefined) => {
  if (!dateString) return ''
  return new Date(dateString).toLocaleDateString(locale.value)
}

const translateRole = (role: string | undefined) => {
  if (!role || typeof role !== 'string') {
    return role || ''
  }
  const lowerRole = role.toLowerCase()
  const translationKey = `roles.${lowerRole}`
  const translated = t(translationKey)
  // If translation doesn't exist, return original role name
  return translated !== translationKey ? translated : role
}

const getRoleClass = (role: string | undefined) => {
  // Handle cases where role might be undefined or null
  if (typeof role !== 'string' || !role) {
    return 'bg-gray-100 text-gray-600'
  }
  const lowerRole = role.toLowerCase()
  if (lowerRole === 'admin') return 'bg-red-100 text-red-800'
  if (lowerRole === 'moderator') return 'bg-yellow-100 text-yellow-800'
  if (lowerRole === 'editor') return 'bg-blue-100 text-blue-800'
  if (lowerRole === 'unconfirmed') return 'bg-gray-100 text-gray-600'
  return 'bg-green-100 text-green-800' // Default for 'user'
}
</script>
