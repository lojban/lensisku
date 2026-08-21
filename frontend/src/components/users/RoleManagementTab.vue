<template>
  <div class="space-y-4 sm:space-y-6">
    <!-- Create New Role Section -->
    <div class="bg-white p-3 sm:p-4 rounded-lg shadow">
      <h2 class="text-lg sm:text-xl font-semibold mb-3 sm:mb-4">
        {{ t('roleManagement.createNewRole') }}
      </h2>

      <div class="flex flex-col gap-3 sm:gap-4">
        <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 items-stretch sm:items-center">
          <Input
            :model-value="newRoleName"
            :placeholder="t('roleManagement.roleNamePlaceholder')"
            class="input-field w-full sm:flex-1"
            @update:model-value="$emit('update:newRoleName', $event)"
          />
          <Button
            variant="create"
            class="w-full sm:w-auto"
            :disabled="!newRoleName.trim() || selectedPermissions.length === 0"
            @click="$emit('createRole')"
          >
            {{ t('roleManagement.createRoleButton') }}
          </Button>
        </div>

        <div v-if="newRoleName.trim()" class="flex flex-col gap-2">
          <label class="font-medium text-sm sm:text-base">{{
            t('roleManagement.selectPermissions')
          }}</label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div
              v-for="perm in availablePermissions"
              :key="perm.name"
              class="flex items-center p-2 rounded cursor-pointer hover:bg-gray-50"
              :class="{ 'bg-gray-100': selectedPermissions.includes(perm.name) }"
              @click="$emit('togglePermission', perm.name)"
            >
              <Checkbox
                :checked="selectedPermissions.includes(perm.name)"
                class="mr-2"
                @click.stop
              />
              <div>
                <div class="font-medium">{{ perm.name }}</div>

                <div class="text-sm text-gray-600">{{ perm.description }}</div>
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

      <div v-for="role in roles" :key="role.name" class="mb-4 sm:mb-6 border-b pb-3 sm:pb-4">
        <div
          class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 sm:gap-4 mb-3 sm:mb-4"
        >
          <h3 class="text-base sm:text-lg font-medium">{{ translateRole(role.name) }}</h3>
          <Button
            v-if="!['admin', 'user', 'editor'].includes(role.name.toLowerCase())"
            variant="delete"
            class="w-full sm:w-auto"
            @click="$emit('deleteRole', role.name)"
          >
            {{ t('roleManagement.deleteRoleButton') }}
          </Button>
        </div>
        <!-- Permissions Section -->
        <div class="sm:pl-4">
          <div
            v-if="availablePermissions.filter((p) => !role.permissions.includes(p.name)).length > 0"
            class="flex flex-col sm:flex-row justify-center items-stretch sm:items-center gap-2 sm:gap-4 mb-3 sm:mb-4"
          >
            <Select
              :model-value="
                selectedPermissionMap[role.name]
                  ? JSON.stringify(selectedPermissionMap[role.name])
                  : ''
              "
              class="input-field w-full sm:flex-1 h-6 py-0"
              :options="[
                { value: '', label: t('roleManagement.selectPermission'), disabled: true },
                ...availablePermissions
                  .filter((p) => !role.permissions.includes(p.name))
                  .map((perm) => ({
                    value: JSON.stringify(perm),
                    label: `${perm.name} - ${perm.description}`,
                  })),
              ]"
              @update:model-value="
                (value) =>
                  $emit('update:selectedPermission', {
                    roleName: role.name,
                    permission: value ? JSON.parse(String(value)) : null,
                  })
              "
            />
            <Button
              variant="create"
              class="w-full sm:w-auto"
              :disabled="!selectedPermissionMap[role.name]"
              @click="$emit('addPermission', role.name)"
            >
              {{ t('roleManagement.addPermissionButton') }}
            </Button>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div
              v-for="perm in availablePermissions.filter((p) => role.permissions.includes(p.name))"
              :key="perm.name"
              class="bg-gray-50 p-2 rounded flex justify-between items-center hover:bg-gray-100"
            >
              <div>
                <div class="font-medium">{{ perm.name }}</div>

                <div class="text-sm text-gray-600">{{ perm.description }}</div>
              </div>
              <Button
                v-if="role.name.toLowerCase() !== 'admin'"
                variant="delete"
                @click="$emit('deletePermission', { roleName: role.name, permission: perm.name })"
              >
                <Trash2 class="w-4 h-4" />
                <span class="sr-only">{{ t('roleManagement.removePermissionButton') }}</span>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Checkbox, Input, Select } from '@packages/ui'
import { Trash2 } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import type { PropType } from 'vue'

const { t } = useI18n()

type PermissionRow = { name: string; description: string }
type RoleRow = { name: string; permissions: string[] }

defineProps({
  roles: { type: Array as PropType<RoleRow[]>, required: true },
  availablePermissions: { type: Array as PropType<PermissionRow[]>, required: true },
  newRoleName: { type: String, required: true },
  selectedPermissions: { type: Array as PropType<string[]>, required: true },
  selectedPermissionMap: { type: Object as PropType<Record<string, unknown>>, required: true }, // Map roleName -> selectedPermission object
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

const translateRole = (role: string) => {
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
