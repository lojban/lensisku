<template>
  <div class="space-y-4 sm:space-y-6">
    <!-- Create New Role Section -->
    <div class="bg-white p-3 sm:p-4 rounded-lg shadow">
      <h2 class="text-lg sm:text-xl font-semibold mb-3 sm:mb-4">
        {{ t('roleManagement.createNewRole') }}
      </h2>
      <div class="flex flex-col gap-3 sm:gap-4">
        <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 items-stretch sm:items-center">
          <input
            :value="newRoleName"
            :placeholder="t('roleManagement.roleNamePlaceholder')"
            class="input-field w-full sm:flex-1"
            @input="$emit('update:newRoleName', $event.target.value)"
          >
          <button
            class="btn-aqua-emerald w-full sm:w-auto"
            :disabled="!newRoleName.trim() || selectedPermissions.length === 0"
            @click="$emit('createRole')"
          >
            {{ t('roleManagement.createRoleButton') }}
          </button>
        </div>

        <div
          v-if="newRoleName.trim()"
          class="flex flex-col gap-2"
        >
          <label class="font-medium text-sm sm:text-base">{{ t('roleManagement.selectPermissions') }}</label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div
              v-for="perm in availablePermissions"
              :key="perm.name"
              class="flex items-center p-2 rounded cursor-pointer hover:bg-gray-50"
              :class="{'bg-gray-100': selectedPermissions.includes(perm.name)}"
              @click="$emit('togglePermission', perm.name)"
            >
              <input
                type="checkbox"
                :checked="selectedPermissions.includes(perm.name)"
                class="mr-2"
                @click.stop
              >
              <div>
                <div class="font-medium">
                  {{ perm.name }}
                </div>
                <div class="text-sm text-gray-600">
                  {{ perm.description }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Roles List -->
    <div class="bg-white p-3 sm:p-4 rounded-lg shadow">
      <h2 class="text-lg sm:text-xl font-semibold mb-3 sm:mb-4">
        {{ t('roleManagement.existingRoles') }}
      </h2>

      <div
        v-for="role in roles"
        :key="role.name"
        class="mb-4 sm:mb-6 border-b pb-3 sm:pb-4"
      >
        <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 sm:gap-4 mb-3 sm:mb-4">
          <h3 class="text-base sm:text-lg font-medium">
            {{ translateRole(role.name) }}
          </h3>
          <button
            v-if="!['admin', 'user', 'editor'].includes(role.name.toLowerCase())"
            class="btn-aqua-red w-full sm:w-auto"
            @click="$emit('deleteRole', role.name)"
          >
            {{ t('roleManagement.deleteRoleButton') }}
          </button>
        </div>

        <!-- Permissions Section -->
        <div class="sm:pl-4">
          <div
            v-if="availablePermissions.filter(p => !role.permissions.includes(p.name)).length > 0"
            class="flex flex-col sm:flex-row justify-center items-stretch sm:items-center gap-2 sm:gap-4 mb-3 sm:mb-4"
          >
            <select
              :value="selectedPermissionMap[role.name]"
              class="input-field w-full sm:flex-1 h-6 py-0"
              @change="$emit('update:selectedPermission', { roleName: role.name, permission: JSON.parse($event.target.value) })"
            >
              <option
                value=""
                disabled
              >
                {{ t('roleManagement.selectPermission') }}
              </option>
              <option
                v-for="perm in availablePermissions.filter(p => !role.permissions.includes(p.name))"
                :key="perm.name"
                :value="JSON.stringify(perm)"
              >
                {{ perm.name }} - {{ perm.description }}
              </option>
            </select>
            <button
              class="btn-aqua-emerald w-full sm:w-auto"
              :disabled="!selectedPermissionMap[role.name]"
              @click="$emit('addPermission', role.name)"
            >
              {{ t('roleManagement.addPermissionButton') }}
            </button>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div
              v-for="perm in availablePermissions.filter(p => role.permissions.includes(p.name))"
              :key="perm.name"
              class="bg-gray-50 p-2 rounded flex justify-between items-center hover:bg-gray-100"
            >
              <div>
                <div class="font-medium">
                  {{ perm.name }}
                </div>
                <div class="text-sm text-gray-600">
                  {{ perm.description }}
                </div>
              </div>
              <button
                v-if="role.name.toLowerCase() !== 'admin'"
                class="btn-delete"
                @click="$emit('deletePermission', { roleName: role.name, permission: perm.name })"
              >
                <Trash2 class="w-4 h-4" />
                <span class="sr-only">{{ t('roleManagement.removePermissionButton') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Trash2 } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

defineProps({
  roles: { type: Array, required: true },
  availablePermissions: { type: Array, required: true },
  newRoleName: { type: String, required: true },
  selectedPermissions: { type: Array, required: true },
  selectedPermissionMap: { type: Object, required: true }, // Map roleName -> selectedPermission object
})

defineEmits([
  'update:newRoleName',
  'update:selectedPermission',
  'createRole',
  'deleteRole',
  'addPermission',
  'deletePermission',
  'togglePermission',
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
</script>
