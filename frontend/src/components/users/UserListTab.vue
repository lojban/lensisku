<template>
  <div>
    <!-- Search and Filter Controls -->
    <div class="bg-white p-4 rounded-lg shadow-sm mb-6">
      <div class="flex flex-wrap gap-4">
        <!-- Search Input -->
        <SearchInput
          :model-value="searchQuery"
          :is-loading="isSearching"
          :placeholder="t('userList.searchPlaceholder')"
          @update:model-value="$emit('update:searchQuery', $event)"
          @keyup.enter="$emit('updateSearch')"
          @clear="$emit('clearSearch')"
        />

        <!-- Role Filter -->
        <div>
          <div class="flex items-center gap-2">
            <label class="text-sm text-gray-600 whitespace-nowrap">{{ t('components.userListTab.roleLabel') }}</label>
            <select
              :value="roleFilter"
              class="input-field flex-1"
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
        </div>

        <!-- Sort Controls -->
        <div>
          <div class="flex items-center gap-2">
            <label class="text-sm text-gray-600 whitespace-nowrap">{{ t('components.userListTab.sortByLabel') }}</label>
            <select
              :value="sortBy"
              class="input-field flex-1"
              @change="$emit('update:sortBy', $event.target.value)"
            >
              <option value="username">
                {{ t('components.userListTab.usernameSort') }}
              </option>
              <option value="realname">
                {{ t('components.userListTab.realNameSort') }}
              </option>
              <option value="created_at">
                {{ t('components.userListTab.createdAtSort') }}
              </option>
            </select>
            <label class="text-sm text-gray-600 whitespace-nowrap">{{ t('components.userListTab.sortOrderLabel') }}</label>
            <select
              :value="sortOrder"
              class="input-field flex-1"
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
    </div>

    <div class="space-y-4 min-h-[400px]">
      <!-- User Cards with hover effects -->
      <div class="grid gap-4">
        <div
          v-for="user in userList"
          :key="user.user_id"
          class="bg-white p-4 rounded-lg border hover:border-blue-300 hover:shadow-sm transition-all duration-200 cursor-pointer"
          role="button"
          tabindex="0"
          @click="$emit('viewUser', user.username)"
          @keyup.enter="$emit('viewUser', user.username)"
        >
          <div class="flex justify-between items-start gap-4">
            <div class="min-w-0 flex-1">
              <h3 class="text-lg font-medium text-blue-600 truncate">
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
              class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
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
