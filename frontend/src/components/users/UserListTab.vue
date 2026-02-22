<template>
  <div class="space-y-6">
    <!-- Search and Filter Controls (compact single row) -->
    <div class="bg-white border border-gray-200 rounded-xl shadow-sm p-3 sm:p-4">
      <div class="flex flex-wrap items-center gap-3 sm:gap-4">
        <!-- Search: compact width -->
        <div class="w-full sm:w-auto sm:min-w-[220px] sm:max-w-[280px] flex-1 sm:flex-initial">
          <SearchInput
            :model-value="searchQuery"
            :is-loading="isSearching"
            :placeholder="t('userList.searchPlaceholder')"
            :show-search-icon="true"
            @update:model-value="$emit('update:searchQuery', $event)"
            @keyup.enter="$emit('updateSearch')"
            @clear="$emit('clearSearch')"
          />
        </div>

        <!-- Role Filter -->
        <div class="flex items-center gap-2 shrink-0">
          <label class="text-sm font-medium text-gray-700 whitespace-nowrap">{{ t('components.userListTab.roleLabel') }}</label>
          <select
            :value="roleFilter"
            class="input-field"
            @change="$emit('update:roleFilter', $event.target.value)"
          >
            <option value="">
              {{ t('components.userListTab.allRoles') }}
            </option>
            <option
              v-for="role in availableRoles"
              :key="role.name"
              :value="role.name"
            >
              {{ translateRole(role.name) }}
            </option>
          </select>
        </div>

        <!-- Sort By -->
        <div class="flex items-center gap-2 shrink-0">
          <label class="text-sm font-medium text-gray-700 whitespace-nowrap">{{ t('components.userListTab.sortByLabel') }}</label>
          <select
            :value="sortBy"
            class="input-field"
            @change="$emit('update:sortBy', $event.target.value)"
          >
            <option value="created_at">
              {{ t('components.userListTab.createdAtSort') }}
            </option>
            <option value="username">
              {{ t('components.userListTab.usernameSort') }}
            </option>
            <option value="realname">
              {{ t('components.userListTab.realNameSort') }}
            </option>
          </select>
        </div>

        <!-- Sort Order -->
        <div class="flex items-center gap-2 shrink-0">
          <label class="text-sm font-medium text-gray-700 whitespace-nowrap">{{ t('components.userListTab.sortOrderLabel') }}</label>
          <select
            :value="sortOrder"
            class="input-field"
            @change="$emit('update:sortOrder', $event.target.value)"
          >
            <option value="asc">
              {{ t('components.userListTab.ascSort') }}
            </option>
            <option value="desc">
              {{ t('components.userListTab.descSort') }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <!-- User list -->
    <div class="min-h-[400px]">
      <!-- Loading state -->
      <div v-if="isLoading && userList.length === 0" class="flex flex-col items-center justify-center py-16">
        <div class="animate-spin rounded-full h-10 w-10 border-2 border-blue-500 border-t-transparent" aria-hidden="true" />
        <p class="mt-3 text-sm text-gray-600">{{ t('userList.loadingUsers') }}</p>
      </div>

      <div v-else class="grid gap-3 sm:gap-4">
        <div
          v-for="user in userList"
          :key="user.user_id"
          class="bg-white p-4 sm:p-5 rounded-xl border border-gray-200 hover:border-blue-400/60 hover:shadow-md transition-all duration-200 cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          role="button"
          tabindex="0"
          @click="$emit('viewUser', user.username)"
          @keyup.enter="$emit('viewUser', user.username)"
        >
          <div class="flex justify-between items-start gap-4">
            <div class="min-w-0 flex-1">
              <h3 class="text-lg font-medium text-blue-600 truncate hover:text-blue-700">
                {{ user.username }}
              </h3>
              <p
                v-if="user.realname"
                class="text-gray-600 text-sm mt-1 truncate"
              >
                {{ user.realname }}
              </p>
            </div>
            <span
              class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium shrink-0"
              :class="getRoleClass(user.role)"
            >
              {{ translateRole(user.role) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <PaginationComponent
      v-if="total > perPage"
      :current-page="currentPage"
      :total-pages="totalPages"
      :total="total"
      :per-page="perPage"
      class="mt-6"
      @prev="$emit('prevPage')"
      @next="$emit('nextPage')"
    />
  </div>
</template>

<script setup>
import { useI18n } from 'vue-i18n'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SearchInput from '@/components/SearchInput.vue'

const { t } = useI18n()

defineProps({
  userList: { type: Array, required: true },
  total: { type: Number, required: true },
  perPage: { type: Number, required: true },
  currentPage: { type: Number, required: true },
  totalPages: { type: Number, required: true },
  availableRoles: { type: Array, required: true },
  isLoading: { type: Boolean, required: true },
  isSearching: { type: Boolean, required: true },
  searchQuery: { type: String, required: true },
  roleFilter: { type: String, required: true },
  sortBy: { type: String, required: true },
  sortOrder: { type: String, required: true },
})

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

const translateRole = (role) => {
  if (!role || typeof role !== 'string') {
    return role || ''
  }
  const lowerRole = role.toLowerCase()
  const translationKey = `roles.${lowerRole}`
  const translated = t(translationKey)
  // If translation doesn't exist, return original role name
  return translated !== translationKey ? translated : role
}

const getRoleClass = (role) => {
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
