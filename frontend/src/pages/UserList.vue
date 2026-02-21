<template>
  <TabbedPageHeader
    :tabs="tabs"
    :active-tab="activeTab"
    :page-title="activeTab === 'users' ? t('userList.users') : t('userList.roleManagement')"
    @tab-click="handleTabClick"
  />

  <!-- Users Tab Content -->
  <UserListTab
    v-if="activeTab === 'users'"
    :user-list="userList"
    :total="total"
    :per-page="perPage"
    :current-page="currentPage"
    :total-pages="totalPages"
    :available-roles="availableRoles"
    :is-loading="isLoading"
    :is-searching="isSearching"
    :search-query="searchQuery"
    :role-filter="roleFilter"
    :sort-by="sortBy"
    :sort-order="sortOrder"
    @update:search-query="searchQuery = normalizeSearchQuery($event); updateSearch()"
    @update:role-filter="roleFilter = $event; updateSearch()"
    @update:sort-by="sortBy = $event; updateSearch()"
    @update:sort-order="sortOrder = $event; updateSearch()"
    @updateSearch="updateSearch"
    @clearSearch="clearSearch"
    @prevPage="prevPage"
    @nextPage="nextPage"
    @viewUser="viewUser"
  />

  <!-- Roles Tab Content -->
  <RoleManagementTab
    v-if="activeTab === 'roles' && hasManageRolesPermission"
    :roles="roles"
    :available-permissions="permissions"
    :new-role-name="newRoleName"
    :selected-permissions="selectedPermissions"
    :selected-permission-map="selectedPermissionMap"
    @update:new-role-name="newRoleName = $event"
    @update:selected-permission="handleUpdateSelectedPermission"
    @createRole="handleCreateRole"
    @deleteRole="handleDeleteRole"
    @addPermission="handleAddPermission"
    @deletePermission="handleDeletePermission"
    @togglePermission="handleTogglePermission"
  />

  <!-- Confirmation Modals (remain in the parent) -->
  <DeleteConfirmationModal
    :show="showDeletePermissionConfirm"
    :title="t('roleManagement.deletePermissionConfirmTitle')"
    :message="t('roleManagement.deletePermissionConfirmMessage', { permission: permissionToDelete.permission, roleName: permissionToDelete.roleName })"
    :is-deleting="isDeletingPermission"
    @confirm="performDeletePermission(permissionToDelete.roleName, permissionToDelete.permission)"
    @cancel="showDeletePermissionConfirm = false"
  />

  <DeleteConfirmationModal
    :show="showDeleteRoleConfirm"
    :title="t('roleManagement.deleteRoleConfirmTitle')"
    :message="t('roleManagement.deleteRoleConfirmMessage', { roleName: roleToDelete })"
    :is-deleting="isDeletingRole"
    @confirm="performDeleteRole"
    @cancel="showDeleteRoleConfirm = false"
  />
</template>

<script setup>
import { Shield, Users } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'

import { listUsers, getRoles, getPermissions, createRole, updateRole, deleteRole } from '@/api.js'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import RoleManagementTab from '@/components/users/RoleManagementTab.vue'
import UserListTab from '@/components/users/UserListTab.vue'
import { useAuth } from '@/composables/useAuth.js'
import { useError } from '@/composables/useError.js'
import { useSeoHead } from '@/composables/useSeoHead'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const router = useRouter()
const route = useRoute()
const auth = useAuth()
const { showError, clearError } = useError()
const { t, locale } = useI18n()

// LocalStorage keys
const STORAGE_KEY_PREFIX = 'userList_'

// State for UserListTab
const userList = ref([])
const total = ref(0)
const isLoading = ref(true)
const isSearching = ref(false)
const searchQuery = ref('')
const offset = ref(parseInt(route.query.offset) || 0)
const perPage = ref(20)
const sortBy = ref('created_at')
const sortOrder = ref('desc')
const roleFilter = ref('')
const availableRoles = ref([])

// Load from localStorage
const loadFromLocalStorage = () => {
  if (typeof window === 'undefined') return
  try {
    const savedSortBy = localStorage.getItem(STORAGE_KEY_PREFIX + 'sortBy')
    const savedSortOrder = localStorage.getItem(STORAGE_KEY_PREFIX + 'sortOrder')
    const savedRoleFilter = localStorage.getItem(STORAGE_KEY_PREFIX + 'roleFilter')
    const savedSearchQuery = localStorage.getItem(STORAGE_KEY_PREFIX + 'searchQuery')
    
    if (savedSortBy) sortBy.value = savedSortBy
    if (savedSortOrder) sortOrder.value = savedSortOrder
    if (savedRoleFilter !== null) roleFilter.value = savedRoleFilter
    if (savedSearchQuery !== null) searchQuery.value = savedSearchQuery
  } catch (e) {
    console.error('Error loading from localStorage:', e)
  }
}

// Save to localStorage
const saveToLocalStorage = () => {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY_PREFIX + 'sortBy', sortBy.value)
    localStorage.setItem(STORAGE_KEY_PREFIX + 'sortOrder', sortOrder.value)
    localStorage.setItem(STORAGE_KEY_PREFIX + 'roleFilter', roleFilter.value)
    localStorage.setItem(STORAGE_KEY_PREFIX + 'searchQuery', searchQuery.value)
  } catch (e) {
    console.error('Error saving to localStorage:', e)
  }
}

// State for RoleManagementTab
const roles = ref([])
const permissions = ref([])
const newRoleName = ref('')
const selectedPermissionMap = ref({}) // Map roleName -> selectedPermission object
const selectedPermissions = ref([]) // For creating new role
const showDeletePermissionConfirm = ref(false)
const permissionToDelete = ref({ roleName: '', permission: '' })
const isDeletingPermission = ref(false)
const showDeleteRoleConfirm = ref(false)
const roleToDelete = ref('')
const isDeletingRole = ref(false)

// Shared State
const activeTab = ref('users') // Default to users tab

// Computed properties
const totalPages = computed(() => Math.ceil(total.value / perPage.value))
const currentPage = computed(() => Math.floor(offset.value / perPage.value) + 1)

const hasManageRolesPermission = computed(() => {
  return auth.state.authorities?.includes('manage_roles') ?? false
}, locale.value)

const tabs = computed(() => {
  const baseTabs = [{ key: 'users', label: t('userList.users'), icon: Users }]
  if (hasManageRolesPermission.value) {
    baseTabs.push({ key: 'roles', label: t('userList.roleManagement'), icon: Shield })
  }
  return baseTabs
})

// Methods for UserListTab
const updateUrlParams = () => {
  router.push({
    query: {
      q: normalizeSearchQuery(searchQuery.value) || undefined,
      offset: offset.value || undefined,
      sort_by: sortBy.value !== 'created_at' ? sortBy.value : undefined,
      sort_order: sortOrder.value !== 'desc' ? sortOrder.value : undefined,
      role: roleFilter.value || undefined,
    },
  })
}

const viewUser = (username) => {
  router.push(`/user/${username}`)
}

const fetchUsers = async () => {
  if (typeof window === 'undefined') return;
  isLoading.value = true
  clearError()

  try {
    const response = await listUsers({
      search: searchQuery.value,
      offset: offset.value,
      per_page: perPage.value,
      sort_by: sortBy.value,
      sort_order: sortOrder.value,
      role: roleFilter.value,
    })

    userList.value = response.data.users
    total.value = response.data.total
  } catch (e) {
    showError(e.response?.data?.error || 'Failed to load users')
    console.error('Error fetching users:', e)
  } finally {
    isLoading.value = false
    isSearching.value = false
  }
}

const updateSearch = () => {
  isSearching.value = true
  offset.value = 0 // Reset offset when searching
  saveToLocalStorage() // Save to localStorage when filters change
  updateUrlParams()
}

const clearSearch = () => {
  searchQuery.value = ''
  offset.value = 0
  updateSearch()
}

const nextPage = () => {
  if (offset.value + perPage.value < total.value) {
    offset.value += perPage.value
    updateUrlParams()
    // fetchUsers() // Fetch is triggered by route watcher
  }
}

const prevPage = () => {
  if (offset.value >= perPage.value) {
    offset.value -= perPage.value
    updateUrlParams()
    // fetchUsers() // Fetch is triggered by route watcher
  }
}

// Methods for RoleManagementTab
const fetchRolesAndPermissions = async () => {
  if (!hasManageRolesPermission.value) return
  isLoading.value = true // Use the same loading flag for simplicity
  try {
    const [rolesRes, permsRes] = await Promise.all([getRoles(), getPermissions()])
    roles.value = rolesRes.data.roles
    permissions.value = permsRes.data.permissions
    // Initialize selectedPermissionMap
    roles.value.forEach(role => {
      if (!selectedPermissionMap.value[role.name]) {
        selectedPermissionMap.value[role.name] = null;
      }
    });
  } catch (error) {
    showError(t('userList.loadRolesError')) // Use translation key
    console.error('Error loading role data:', error)
  } finally {
    isLoading.value = false
  }
}

const handleUpdateSelectedPermission = ({ roleName, permission }) => {
  selectedPermissionMap.value[roleName] = permission;
};

const handleTogglePermission = (permission) => {
  const index = selectedPermissions.value.indexOf(permission)
  if (index === -1) {
    selectedPermissions.value.push(permission)
  } else {
    selectedPermissions.value.splice(index, 1)
  }
}

const handleCreateRole = async () => {
  if (!newRoleName.value.trim() || !selectedPermissions.value.length) return

  try {
    await createRole({ name: newRoleName.value.trim(), permissions: selectedPermissions.value })
    selectedPermissions.value = []
    await fetchRolesAndPermissions() // Refresh roles
    newRoleName.value = ''
  } catch (error) {
    showError(t('userList.createRoleError')) // Use translation key
    console.error('Error creating role:', error)
  }
}

const handleAddPermission = async (roleName) => {
  const selectedPermission = selectedPermissionMap.value[roleName];
  if (!selectedPermission) return

  try {
    const role = roles.value.find(r => r.name === roleName)
    if (!role) return
    const newPermissions = [...role.permissions, selectedPermission.name]
    await updateRole(roleName, { permissions: newPermissions })
    await fetchRolesAndPermissions() // Refresh roles
    selectedPermissionMap.value[roleName] = null; // Reset selection for this role
  } catch (error) {
    showError(t('userList.addPermissionError')) // Use translation key
    console.error('Error adding permission:', error)
  }
}

const handleDeletePermission = ({ roleName, permission }) => {
  permissionToDelete.value = { roleName, permission }
  showDeletePermissionConfirm.value = true
}

const performDeletePermission = async (roleName, permission) => {
  try {
    isDeletingPermission.value = true
    const role = roles.value.find(r => r.name === roleName)
    if (!role) return
    const newPermissions = role.permissions.filter(p => p !== permission)
    await updateRole(roleName, { permissions: newPermissions })
    await fetchRolesAndPermissions() // Refresh roles
  } catch (error) {
    showError(t('userList.removePermissionError')) // Use translation key
    console.error('Error removing permission:', error)
  } finally {
    isDeletingPermission.value = false
    showDeletePermissionConfirm.value = false
    permissionToDelete.value = { roleName: '', permission: '' }
  }
}

const handleDeleteRole = (roleName) => {
  roleToDelete.value = roleName
  showDeleteRoleConfirm.value = true
}

const performDeleteRole = async () => {
  if (['admin', 'user'].includes(roleToDelete.value.toLowerCase())) {
    alert(t('roleManagement.systemRoleDeleteError'))
    return
  }

  try {
    isDeletingRole.value = true
    await deleteRole(roleToDelete.value)
    await fetchRolesAndPermissions() // Refresh roles
    newRoleName.value = ''
    selectedPermissions.value = []
  } catch (error) {
    showError(t('userList.deleteRoleError')) // Use translation key
    console.error('Error deleting role:', error)
  } finally {
    isDeletingRole.value = false
    showDeleteRoleConfirm.value = false
    roleToDelete.value = ''
  }
}

// Shared Methods
const handleTabClick = (tabKey) => {
  activeTab.value = tabKey
  if (tabKey === 'roles') {
    fetchRolesAndPermissions()
  } else {
    fetchUsers() // Fetch users when switching back
  }
}

// Sync URL parameters with state
const syncFromRoute = () => {
  // URL params take precedence over localStorage
  if (route.query.q !== undefined) {
    searchQuery.value = normalizeSearchQuery(route.query.q || '')
  }
  offset.value = parseInt(route.query.offset) || 0
  if (route.query.sort_by !== undefined) {
    sortBy.value = route.query.sort_by || 'created_at'
  }
  if (route.query.sort_order !== undefined) {
    sortOrder.value = route.query.sort_order || 'desc'
  }
  if (route.query.role !== undefined) {
    roleFilter.value = route.query.role || ''
  }
}

// Watch for route changes
watch(
  () => route.query,
  (newQuery, oldQuery) => {
    if (JSON.stringify(newQuery) !== JSON.stringify(oldQuery)) {
      syncFromRoute()
      if (activeTab.value === 'users') {
        fetchUsers()
      }
      // Roles data is fetched on tab click or mount
    }
  },
  { deep: true, immediate: true } // immediate: true to run on mount
)

// Reactive page title
const pageTitle = ref('Users')
useSeoHead({ title: pageTitle }, locale.value)

// Update title when search parameters change or tab changes
watch(
  [activeTab, searchQuery, sortBy, sortOrder, roleFilter],
  () => {
    if (activeTab.value === 'roles') {
      pageTitle.value = t('userList.roleManagement')
    } else {
      const titleParts = []
      if (searchQuery.value) {
        titleParts.push(`${t('searchForm.modes.dictionary')}: "${searchQuery.value}"`) // Assuming dictionary search context
      }
      if (roleFilter.value) {
        titleParts.push(`${t('profile.role')}: ${roleFilter.value}`)
      }
      if (sortBy.value !== 'created_at' || sortOrder.value !== 'desc') {
        titleParts.push(`${t('userList.sortByLabel')} ${t(`userList.sortBy.${sortBy.value}`)} ${sortOrder.value === 'asc' ? t('userList.sortOrder.asc') : t('userList.sortOrder.desc')}`)
      }
      pageTitle.value = titleParts.length ? `${t('userList.users')} - ${titleParts.join(' | ')}` : t('userList.users')
    }
  },
  { immediate: true }
)

onMounted(async () => {
  // Load from localStorage first (will be overridden by URL params if they exist)
  loadFromLocalStorage()
  // Then sync from route (URL params take precedence)
  syncFromRoute()
  // Save back to localStorage after syncing (to persist URL params if they exist)
  saveToLocalStorage()
  try {
    const rolesResponse = await getRoles()
    availableRoles.value = rolesResponse.data.roles
  } catch (error) {
    console.error('Failed to fetch roles:', error)
  }
  // Fetch initial data based on the active tab
  if (activeTab.value === 'users') {
    fetchUsers()
  } else if (activeTab.value === 'roles') {
    fetchRolesAndPermissions()
  }
})
</script>
