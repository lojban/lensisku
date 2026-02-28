<template>
  <!-- Loading State -->
  <LoadingSpinner
    v-if="isLoading"
    class="py-8"
  />

  <!-- Profile Content -->
  <div v-else>
    <!-- Header -->
    <div class="flex flex-col lg:flex-row justify-between items-center gap-4 mb-6">
      <h2 class="text-2xl font-bold text-gray-800 text-center sm:text-left flex-1 min-w-[200px]">
        {{ isOwnProfile ? t('profile.yourProfile') : t('profile.userProfile', { username: profileData.username }) }}
      </h2>
      <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-center items-center">
        <!-- Language Selector -->
        <div v-if="isOwnProfile" class="relative group">
          <select
            :value="locale"
            class="input-field appearance-none !h-6 !py-0 !pr-8 !text-xs"
            @change="switchLanguage"
          >
            <option
              v-for="loc in availableLocales"
              :key="`locale-${loc}`"
              :value="loc"
            >
              {{ loc.toUpperCase() }}
            </option>
          </select>
          <ChevronDown class="h-4 w-4 text-gray-400 absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none" />
        </div>
        <!--
        <RouterLink
          v-if="isOwnProfile"
          to="/balance"
          class="btn-aqua-rose"
        >
          <Wallet class="h-4 w-4" />
          {{ t('profile.balance') }}
        </RouterLink>
        -->
        <!-- Aqua button group -->
        <div class="flex">
          <RouterLink
            v-if="isOwnProfile"
            to="/change-password"
            class="btn-aqua-orange btn-aqua-group-item"
          >
            <KeyRound class="h-4 w-4" />
            {{ t('profile.changePassword') }}
          </RouterLink>
          <button
            v-if="isOwnProfile && !isEditing"
            class="btn-aqua-yellow btn-aqua-group-item"
            @click="toggleEdit"
          >
            <Pencil class="h-4 w-4" />
            {{ t('profile.editProfile') }}
          </button>
          </div>
          <div class="flex">
          <RouterLink
            :to="`/user/${profileData.username}/activity?tab=definitions`"
            class="btn-aqua-purple btn-aqua-group-item"
          >
            <Activity class="h-4 w-4" />
            {{ t('profile.viewActivity') }}
          </RouterLink>
          <button
            v-if="isOwnProfile && auth.state.isLoggedIn"
            class="btn-aqua btn-aqua-group-item"
            @click="auth.logout()"
          >
            <LogOut class="h-4 w-4" />
            {{ t('nav.logout') }}
          </button>
        </div>

        <!-- Role Assignment Section -->
        <div
          v-if="canAssignRoles && !isOwnProfile"
          class="flex gap-2 items-center form-group"
        >
          <select
            v-model="selectedRole"
            class="input-field h-6 py-0"
          >
            <option
              v-for="role in assignableRoles"
              :key="role.name"
              :value="role.name"
            >
              {{ translateRole(role.name) }}
            </option>
          </select>
          <button
            class="btn-aqua-emerald"
            :disabled="isAssigningRole"
            @click="performAssignRole"
          >
            {{ isAssigningRole ? t('profile.assigning') : t('profile.assignRole') }}
          </button>
        </div>
      </div>
    </div>

    <!-- View Mode -->
    <div
      v-if="!isEditing"
      class="bg-white p-4 rounded-lg border space-y-4"
    >
      <div class="">
        <!-- Profile Image -->
        <div class="mb-6 flex flex-col items-center">
          <!-- Skeleton for View Mode -->
          <div v-show="isViewProfileImageLoading" class="w-32 h-32 rounded-full bg-gray-200 animate-pulse border-4 border-white shadow-lg"></div>
          <!-- Actual Image for View Mode -->
          <img
            v-show="!isViewProfileImageLoading && profileData.has_profile_image"
            :src="profileImageUrl"
            :alt="`${profileData.username}'s profile picture`"
            class="w-32 h-32 rounded-full object-cover border-4 border-white shadow-lg"
            @load="handleViewProfileImageLoad"
            @error="handleViewProfileImageError"
          >
          <!-- Placeholder for View Mode (No Image or Error) -->
          <div
            v-show="!isViewProfileImageLoading && !profileData.has_profile_image"
            class="w-32 h-32 rounded-full bg-gray-200 flex items-center justify-center text-gray-400 border-4 border-white shadow-lg"
          >
            <User class="h-16 w-16" />
          </div>
        </div>

        <!-- Profile Info -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 w-full">
          <!-- Left Column -->
          <div class="space-y-4">
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.username') }}</span>
              <p class="mt-1 text-gray-900">
                {{ profileData.username }}
              </p>
            </div>
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.role') }}</span>
              <p class="mt-1 text-gray-900">
                {{ translateRole(profileData.role) }}
              </p>
            </div>
            <div v-if="isOwnProfile || profileData.email">
              <span class="text-sm font-medium text-gray-500">{{ t('profile.email') }}</span>
              <p class="mt-1 text-gray-900">
                {{ profileData.email }}
              </p>
            </div>
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.realName') }}</span>
              <p class="mt-1 text-gray-900">
                {{ profileData.realname || t('profile.notSet') }}
              </p>
            </div>
          </div>

          <!-- Right Column -->
          <div class="space-y-4">
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.url') }}</span>
              <p class="mt-1">
                <a
                  v-if="profileData.url"
                  :href="profileData.url"
                  target="_blank"
                  class="text-blue-600 hover:underline"
                >{{ profileData.url }}</a>
                <span
                  v-else
                  class="text-gray-900"
                >{{ t('profile.notSet') }}</span>
              </p>
            </div>
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.personalInfo') }}</span>
              <p class="mt-1 text-gray-900">
                <LazyMathJax
                  :content="profileData.personal || t('profile.notSet')"
                  class="inline"
                />
              </p>
            </div>
            <div>
              <span class="text-sm font-medium text-gray-500">{{ t('profile.memberSince') }}</span>
              <p class="mt-1 text-gray-900">
                {{ memberSince }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Mode -->
    <form
      v-else
      class="bg-white p-4 rounded-lg border space-y-4"
      @submit.prevent="performUpdateProfile"
    >
      <!-- Integrated Avatar Upload UI -->
      <div class="flex flex-col items-center mb-6">
        <div class="relative group w-32 h-32">
          <!-- Skeleton for Edit Mode -->
          <div v-show="isEditProfileImageLoading" class="w-32 h-32 rounded-full bg-gray-200 animate-pulse border-4 border-white shadow-lg"></div>
          <!-- Avatar Image or Placeholder for Edit Mode -->
          <img
            v-show="!isEditProfileImageLoading && currentImageUrl"
            :key="currentImageUrl" 
            :src="currentImageUrl"
            :alt="`${profileData.username}'s profile picture`"
            class="w-32 h-32 rounded-full object-cover border-4 border-white shadow-lg"
            @load="handleEditProfileImageLoad"
            @error="handleEditProfileImageError"
          >
          <div
            v-show="!isEditProfileImageLoading && !currentImageUrl"
            class="w-32 h-32 rounded-full bg-gray-200 flex items-center justify-center text-gray-400 border-4 border-white shadow-lg"
          >
            <User class="h-16 w-16" />
          </div>

          <!-- Loading/Progress Overlay -->
          <div
            v-if="isImageUploading"
            class="absolute inset-0 rounded-full bg-black bg-opacity-50 flex items-center justify-center"
          >
            <svg class="w-16 h-16 transform -rotate-90" viewBox="0 0 100 100">
              <circle cx="50" cy="50" r="45" fill="none" stroke="#4B5563" stroke-width="8" />
              <circle
                cx="50" cy="50" r="45" fill="none" stroke="#3B82F6" stroke-width="8"
                :stroke-dasharray="circumference"
                :stroke-dashoffset="circumference - (uploadProgress / 100) * circumference"
                class="transition-all duration-300"
              />
            </svg>
            <span class="absolute text-white text-sm font-medium">{{ Math.round(uploadProgress) }}%</span>
          </div>


          <!-- Action Buttons (Always visible below when not uploading) -->
          <div
            v-if="!isImageUploading"
            class="absolute -bottom-4 left-1/2 transform -translate-x-1/2 flex gap-3"
          >
            <label
              class="cursor-pointer p-2 bg-white border border-gray-300 rounded-full text-blue-600 hover:bg-blue-50 transition-all shadow-md"
              :title="t('profile.uploadNewPhoto')"
            >
              <input type="file" class="hidden" accept="image/*" @change="handleFileChange">
              <Camera class="h-5 w-5" />
            </label>
            <button
              v-if="hasImage"
              class="p-2 bg-white border border-gray-300 rounded-full text-red-600 hover:bg-red-50 transition-all shadow-md"
              :title="t('profile.removePhoto')"
              @click.stop="handleImageRemove"
            >
              <Trash2 class="h-5 w-5" />
            </button>
          </div>
        </div>

        <!-- Spacer to prevent overlap with buttons -->
        <div class="h-6" />

        <!-- Upload Error Message -->
        <p v-if="imageUploadError" class="mt-2 text-sm text-red-600">
          {{ imageUploadError }}
        </p>
      </div>
      <!-- End Integrated Avatar Upload UI -->

      <!-- Rest of the form fields -->
      <div>
        <label class="block text-sm font-medium text-gray-700">{{ t('profile.username') }}</label>
        <input 
          v-model="editForm.username"
          type="text"
          class="input-field w-full"
          :disabled="isUpdating"
        >
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700">{{ t('profile.realName') }}</label>
        <input
          v-model="editForm.realname"
          type="text"
          class="input-field w-full"
        >
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700">{{ t('profile.url') }}</label>
        <input
          v-model="editForm.url"
          type="url"
          class="input-field w-full"
        >
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700">{{ t('profile.personalInfo') }}</label>
        <textarea
          v-model="editForm.personal"
          rows="4"
          class="textarea-field"
        />
      </div>
      <div
        v-if="updateSuccess"
        class="text-green-600 text-sm"
      >
        {{ t('profile.updateSuccess') }}
      </div>

      <div class="mt-6 flex justify-end gap-3">
        <button
          type="button"
          class="btn-cancel"
          @click="toggleEdit"
        >
          {{ t('profile.cancel') }}
        </button>
        <button
          type="submit"
          :disabled="isUpdating || isImageUploading"
          class="btn-update"
        >
          {{ isUpdating ? t('profile.saving') : t('profile.saveChanges') }}
        </button>
      </div>
    </form>

    <!-- Role Assigned ToastFloat -->
    <ToastFloat
      :show="showToast"
      :message="t('profile.roleAssignedSuccess')"
      type="success"
    />
  </div>
</template>

<script setup>
import { jwtDecode } from 'jwt-decode'
import { Wallet, KeyRound, Activity, Pencil, User, ChevronDown, Camera, Trash2, LogOut } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter , RouterLink, useRoute } from 'vue-router'

import {
  updateProfile,
  getUserProfile,
  updateProfileImage,
  removeProfileImage,
  getProfileImage,
  assignRole,
  getRoles
} from '@/api'

import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ToastFloat from '@/components/ToastFloat.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { locale, availableLocales, t } = useI18n()

// State
const { showError, clearError } = useError()
const isLoading = ref(true)
const isEditing = ref(false)
const isUpdating = ref(false)

// --- Profile Image Loading States & Handlers ---
const isViewProfileImageLoading = ref(false);
const handleViewProfileImageLoad = () => { isViewProfileImageLoading.value = false; };
const handleViewProfileImageError = () => {
  console.error('Profile image failed to load (view mode).');
  isViewProfileImageLoading.value = false;
};

const isEditProfileImageLoading = ref(false);
const handleEditProfileImageLoad = () => { isEditProfileImageLoading.value = false; };
const handleEditProfileImageError = () => {
  console.error('Profile image failed to load (edit mode).');
  isEditProfileImageLoading.value = false;
  currentImageUrl.value = null; // Keep original behavior for edit mode
};
// --- End Profile Image Loading States & Handlers ---

const isImageUploading = ref(false) // Keep track of upload state
const imageUploadError = ref('')
const updateSuccess = ref(false)
const memberSince = ref('')
const currentImageUrl = ref(null) // URL for display
const uploadProgress = ref(0) // For progress indicator
const circumference = computed(() => 2 * Math.PI * 45) // For circular progress

const profileData = ref({
  username: '',
  email: '',
  realname: '',
  url: '',
  personal: '',
  user_id: '',
})

const editForm = ref({
  username: '',
  realname: '',
  url: '',
  personal: '',
})

// Computed
useSeoHead({ title: computed(() => profileData.value?.username) }, locale.value)

const hasImage = computed(() => profileData.value.has_profile_image)

const isOwnProfile = computed(() => {
  return !route.params.username || route.params.username === auth.state.username
})

const profileImageUrl = computed(() => {
  if (profileData.value.has_profile_image) {
    return getProfileImage(profileData.value.username)
  }
  return null
})

// Methods
const switchLanguage = (event) => {
  if (typeof window === 'undefined') return;

  const newLocale = event.target.value
  if (availableLocales.includes(newLocale)) {
    locale.value = newLocale
    localStorage.setItem('selectedLocale', newLocale) // Store preference
    // Construct the new path by replacing/adding the locale prefix
    const currentPath = route.path;
    const pathSegments = currentPath.split('/').filter(segment => segment !== ''); // Split and remove empty segments (e.g., leading '/')

    // Check if the first segment is a known locale (assuming availableLocales is accessible in this scope)
    if (pathSegments.length > 0 && availableLocales.includes(pathSegments[0])) {
      // Replace the existing locale prefix
      pathSegments[0] = newLocale;
    } else {
      pathSegments.unshift(newLocale);
    }

    // Reconstruct the path, ensuring it starts with a '/'
    const newPath = '/' + pathSegments.join('/');

    // Update the URL using the new path and preserving query parameters
    router.push({ path: newPath, query: route.query });
    // Optionally, you might want to reload the page or trigger a data refresh
    // window.location.reload(); // Uncomment if a full reload is desired
  }
}

const fetchProfileImageUrl = async () => {
  // Always try to fetch, rely on cache or get updated image
  if (profileData.value.username) {
    try {
      isEditProfileImageLoading.value = true; // Set loading before changing src
      // Append timestamp to bypass cache if needed, or rely on backend/browser caching
      currentImageUrl.value = getProfileImage(profileData.value.username) + `?t=${Date.now()}`
    } catch (err) {
      console.error('Error loading profile image for edit:', err)
      currentImageUrl.value = null;
      isEditProfileImageLoading.value = false; // Error setting it, so not loading
      // Don't nullify if fetch fails, might still have old image
      // currentImageUrl.value = null;
    }
  } else {
    currentImageUrl.value = null;
    isEditProfileImageLoading.value = false;
  }
}

// --- Integrated Image Upload Logic ---
const handleFileChange = async (event) => {
  const file = event.target.files[0]
  if (!file) return

  clearError()
  imageUploadError.value = ''
  uploadProgress.value = 0

  if (!file.type.startsWith('image/')) {
    imageUploadError.value = t('profile.uploadError.invalidType')
    return
  }
  if (file.size > 5 * 1024 * 1024) { // 5MB limit
    imageUploadError.value = t('profile.uploadError.tooLarge')
    return
  }

  try {
    isImageUploading.value = true

    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        const base64Data = e.target.result.split(',')[1]
        const imageData = { data: base64Data, mime_type: file.type }

        isEditProfileImageLoading.value = true; // Set loading for preview
        // Show preview immediately
        currentImageUrl.value = URL.createObjectURL(file) // Temporary preview

        // Simulate upload progress
        let progress = 0
        const progressInterval = setInterval(() => {
          progress += 10
          if (progress > 90) clearInterval(progressInterval)
          uploadProgress.value = Math.min(progress, 90)
        }, 100)

        // Perform the actual upload
        await updateProfileImage(imageData)

        // Complete progress and update state
        uploadProgress.value = 100
        profileData.value.has_profile_image = true
        updateSuccess.value = true
        await fetchProfileImageUrl() // Fetch the potentially updated image URL from server

        setTimeout(() => {
          isImageUploading.value = false
          uploadProgress.value = 0
        }, 500)

      } catch (err) {
        console.error('Upload error:', err)
        imageUploadError.value = err.response?.data?.message || t('profile.uploadError.uploadFailed')
        isImageUploading.value = false
        isEditProfileImageLoading.value = false; // Error during preview setup or upload
        uploadProgress.value = 0
        // Revert preview if upload failed? Or keep it? For now, keep preview.
        // await fetchProfileImageUrl(); // Re-fetch original if needed
      }
    }
    reader.onerror = () => {
      imageUploadError.value = t('profile.uploadError.readError')
      isImageUploading.value = false
      isEditProfileImageLoading.value = false; // Error reading file
      uploadProgress.value = 0
    }
    reader.readAsDataURL(file)

  } catch (err) {
    imageUploadError.value = t('profile.uploadError.uploadFailed')
    isImageUploading.value = false
    uploadProgress.value = 0
  }
}

const handleImageRemove = async () => {
  if (!confirm(t('profile.removeConfirm'))) return

  try {
    isImageUploading.value = true // Use same flag to disable buttons
    imageUploadError.value = ''
    await removeProfileImage()
    profileData.value.has_profile_image = false
    currentImageUrl.value = null // Clear the image URL
    isEditProfileImageLoading.value = false; // No image, so not loading
    updateSuccess.value = true
  } catch (err) {
    imageUploadError.value = err.response?.data?.message || 'Failed to remove image.'
  } finally {
    isImageUploading.value = false
  }
}
// --- End Integrated Image Upload Logic ---

const decodedToken = computed(() => {
  if (typeof window === 'undefined') return;

  const token = localStorage.getItem('accessToken')
  if (token) {
    try {
      return jwtDecode(token)
    } catch (e) {
      console.error('Error decoding token:', e)
      return null
    }
  }
  return null
})

const decodedAuthorities = computed(() => decodedToken.value?.authorities || [])
const currentUserId = computed(() => decodedToken.value?.sub || null)

// Role assignment state
const selectedRole = ref('')
const isAssigningRole = ref(false)
const assignableRoles = ref([])

// Check if current user can assign roles based on permissions
const canAssignRoles = computed(() => {
  return decodedAuthorities.value?.includes('manage_roles')
    && !isOwnProfile.value
})

// Assign role method
const showToast = ref(false)

const performAssignRole = async () => {
  if (!selectedRole.value || !profileData.value || !currentUserId.value) return

  try {
    isAssigningRole.value = true
    clearError()
    await assignRole(profileData.value.user_id, selectedRole.value)
    // Update profile data
    profileData.value.role = selectedRole.value
    updateSuccess.value = true
    showToast.value = true
    setTimeout(() => {
      showToast.value = false
    }, 3000)
  } catch (err) {
    showError(err.response?.data?.message || 'Failed to assign role')
  } finally {
    isAssigningRole.value = false
  }
}

const capitalizeFirstLetter = (str) => str.charAt(0).toUpperCase() + str.substring(1)

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

const fetchProfileData = async () => {
  isLoading.value = true
  clearError()

  const token = decodedToken.value

  try {
    let response,
      username = route.params.username,
      email

    if (isOwnProfile.value) {
      if (token) {
        // For own profile, prefer route parameter username (which might be updated)
        // over token username (which might be stale)
        if (route.params.username) {
          username = route.params.username
        } else {
          username = token.username || 'N/A'
        }
        email = token.email
      }
    }

    try {
      response = await getUserProfile(username)
    } catch (profileErr) {
      // If profile lookup fails and this is own profile, try refreshing token
      if (isOwnProfile.value && auth && auth.refreshAccessToken) {
        console.log('Profile lookup failed, attempting token refresh...')
        const refreshSuccess = await auth.refreshAccessToken()
        if (refreshSuccess) {
          // Retry with potentially updated token
          const newToken = decodedToken.value
          if (newToken && newToken.username) {
            response = await getUserProfile(newToken.username)
          } else {
            throw profileErr // Re-throw if refresh didn't help
          }
        } else {
          throw profileErr // Re-throw if refresh failed
        }
      } else {
        throw profileErr // Re-throw for non-own profiles or if no auth
      }
    }

    const {
      realname,
      url,
      personal,
      join_date,
      has_profile_image,
      user_id,
      role,
    } = response.data

    profileData.value = {
      ...profileData.value,
      user_id,
      realname,
      url,
      personal,
      username,
      email,
      has_profile_image,
      role,
    }

    // Initialize selected role with current user's role
    selectedRole.value = capitalizeFirstLetter(role)

    await fetchProfileImageUrl()

    memberSince.value = join_date ? new Date(join_date).toLocaleDateString(locale.value) : 'N/A'
    editForm.value = { realname, url, personal }


    // Load assignable roles if we have permission
    if (canAssignRoles.value) {
      try {
        const rolesRes = await getRoles()
        assignableRoles.value = rolesRes.data.roles.map((role=>({...role, name: capitalizeFirstLetter(role.name)})))
      } catch (err) {
        console.error('Failed to fetch assignable roles:', err)
      }
    }

  } catch (err) {
    showError('Error loading profile data')
    console.error('Error loading profile:', err)
  } finally {
    isLoading.value = false
  }
}

const toggleEdit = () => {
  if (!isOwnProfile.value) return
  isEditing.value = !isEditing.value
  if (isEditing.value) {
    editForm.value = {
      username: profileData.value.username,
      realname: profileData.value.realname,
      url: profileData.value.url,
      personal: profileData.value.personal,
    }
  }
  clearError()
  updateSuccess.value = false
}

const performUpdateProfile = async () => {
  if (!isOwnProfile.value) return
  if (isImageUploading.value) return

  try {
    isUpdating.value = true
    updateSuccess.value = false // Reset success message on new attempt
    clearError()
    updateSuccess.value = false

    const response = await updateProfile(editForm.value)

    // Check if we got a new token (username was changed)
    if (response.data && response.data.new_token) {
      // Update the authentication state with the new token
      const newToken = response.data.new_token
      localStorage.setItem('accessToken', newToken)
      
      // Update the auth state
      if (auth && auth.state) {
        auth.state.accessToken = newToken
        // Decode the new token to get updated username
        try {
          const decoded = jwtDecode(newToken)
          auth.state.username = decoded.username
          localStorage.setItem('username', decoded.username)
        } catch (e) {
          console.error('Error decoding new token:', e)
        }
      }
    }

    profileData.value = {
      ...profileData.value,
      // Only update username in local state if it was actually changed
      // The backend handles the actual update logic
      ...(editForm.value.username !== profileData.value.username && {
        username: editForm.value.username,
      }),
      ...editForm.value,
    }

    updateSuccess.value = true
    isEditing.value = false
  } catch (err) {
    showError(err.response?.data?.error || 'Error updating profile')
    // If the error is specifically about the username being taken
    if (err.response?.data?.error === 'Username already taken') {
      showError(t('profile.usernameTakenError', 'Username already taken. Please choose another.'));
    }
  } finally {
    isUpdating.value = false
  }
}

// Watch for route changes
watch(
  () => route.params.username,
  () => {
    fetchProfileData()
  }
)

watch(() => profileData.value.has_profile_image, (hasImage) => {
  if (hasImage) {
    isViewProfileImageLoading.value = true;
  } else {
    isViewProfileImageLoading.value = false;
  }
}, { immediate: true });

// Initialize

onMounted(() => {
  // The global router guard (setupRouterGuards in router/index.ts)
  // handles authentication checks. By the time onMounted is called,
  // auth.state.isLoggedIn (and other auth state like username) should be stable
  // if the navigation was allowed by the guard.
  fetchProfileData();
})
</script>
