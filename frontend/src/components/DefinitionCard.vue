<template>
  <div :class="[
    'w-full overflow-visible relative',
    !disableBorder
      ? 'bg-white border rounded-lg hover:border-blue-300 transition-colors shadow hover:shadow-none'
      : '',
  ]" :data-definition-id="definitionId">
    <!-- Header Section -->
    <div :class="[disableBorder ? '' : 'p-4']">
      <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
        <!-- Word and Type Info -->
        <div class="flex-1 w-full space-y-3">
          <!-- Compact Header Layout -->
          <div class="w-full">
            <!-- Main Content -->
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div class="w-auto flex items-center gap-2 flex-wrap min-w-0">
                <h2 class="text-base font-semibold truncate flex-shrink-0 min-w-0 max-w-full"
                  :class="definition.definitionid ? 'text-blue-700 hover:text-blue-800 hover:underline' : 'text-gray-800'">
                  <template v-if="definition.definitionid">
                    <template v-if="isValsiTruncated">
                      <span
                        class="cursor-pointer"
                        :title="t('components.definitionCard.clickToSeeFullWord')"
                        @click="showValsiModal = true">
                        {{ displayedValsi }}
                        <sup v-if="showDefinitionNumber" class="italic font-medium text-gray-600">#</sup>
                      </span>
                    </template>
                    <RouterLink
                      v-else
                      :to="valsiDefinitionLink">
                      {{ definition.valsiword ?? definition.word }}
                      <sup v-if="showDefinitionNumber" class="italic font-medium text-gray-600">
                        #
                      </sup>
                    </RouterLink>
                  </template>
                  <template v-else>
                    <span class="truncate block">{{ displayedFreeContent }}</span>
                  </template>
                </h2>
                <span v-if="definition.type_name && props.showWordType"
                  class="px-2 py-1 text-xs font-medium rounded-full" :class="getTypeClass(definition.type_name)">
                  {{ t(`wordTypes.${definition.type_name.replace(/'/g, 'h').replace(/ /g, '-')}`) }}
                </span>
                <RouterLink v-if="definition.selmaho" :to="{ path: '/', query: selmahoLinkQuery }"
                  class="inline-flex items-center px-2 py-1 text-xs font-medium bg-purple-100 text-purple-700 rounded-md justify-center sm:justify-start hover:bg-purple-200 hover:text-purple-800 transition-colors min-w-0 max-w-full truncate"
                  :title="definition.selmaho.length > MAX_VALSI_DISPLAY_LENGTH ? definition.selmaho : undefined"
                  @click.stop>
                  {{ t('components.definitionCard.selmaoLabel') }} {{ displayedSelmaho }}
                </RouterLink>
                <span v-if="definition.rafsi"
                  class="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-700 rounded-full">
                  {{ definition.rafsi }}
                </span>
                <span :title="t('components.definitionCard.flashcardIndicatorTitle')" class="cursor-pointer"
                  @click.stop="$emit('edit-item')">
                  <GalleryHorizontalIcon v-if="showEditButton && flashcard"
                    class="h-4 w-4 text-purple-600 hover:text-purple-800" />
                </span>
                <AudioPlayer v-if="definition.sound_url && props.showAudio" :url="definition.sound_url"
                  class="shrink-0" />
              </div>
              <div class="flex items-center gap-2 flex-wrap">
                <RouterLink v-if="!disableDiscussionButton" :to="`/valsi/${definition.valsiword ?? definition.word}`"
                  class="btn-empty">
                  <MessageSquarePlus v-if="definition.comment_count === 0" class="h-4 w-4" />
                  <MessageSquareMore v-else class="h-4 w-4" />
                  <span v-if="definition.comment_count"
                    class="bg-gray-100 px-2 py-0.5 rounded-full text-xs font-medium">
                    {{ definition.comment_count }}
                  </span>
                  <span v-else class="hidden sm:inline">{{ t('components.definitionCard.discussButton') }}</span>
                </RouterLink>

                <VoteButtons v-if="props.showVoteButtons && definition.definitionid"
                  :definition-id="definition.definitionid" :initial-score="definition.score"
                  :initial-user-vote="definition.user_vote" @vote-change="() => { }" />
              </div>

              <div class="flex items-center gap-2 flex-wrap">
                <button v-if="showEditButton" class="btn-empty" @click.stop="$emit('edit-item')"
                  :title="t('components.definitionCard.editItemTitle')">
                  <Pencil class="h-4 w-4" />
                  <span class="sr-only">{{ t('components.definitionCard.editButton') }}</span>
                </button>
                <ClipboardButton :content="(
                  definition.definition +
                  (definition.notes ? ' Notes: ' + definition.notes : '')
                ).trim()
                  " :title="t('components.definitionCard.copyTitle')" />
                <CollectionWidget v-if="auth.state.isLoggedIn && definition.definitionid"
                  :definition-id="definition.definitionid" :word="definition.valsiword ?? definition.word"
                  :external-collections="collections" @collection-updated="updateCollections" />
              </div>
              <div v-if="showReorderControls"
                class="flex flex-wrap gap-2 md:gap-0 justify-end sm:justify-end w-full sm:w-auto flex-none"
                role="group">
                <button :disabled="isFirstItem || isReordering" class="btn-group-item btn-empty"
                  :title="t('components.definitionCard.moveUpTitle')" @click.stop="$emit('move-up')">
                  <ArrowUp class="h-4 w-4" />
                  <span class="sr-only">{{ t('components.definitionCard.moveUpTitle') }}</span>
                </button>

                <button :disabled="isLastItem || isReordering" class="btn-group-item btn-empty"
                  :title="t('components.definitionCard.moveDownTitle')" @click.stop="$emit('move-down')">
                  <ArrowDown class="h-4 w-4" />
                  <span class="sr-only">{{ t('components.definitionCard.moveDownTitle') }}</span>
                </button>

                <button class="btn-group-item btn-empty hover:text-red-600"
                  :title="t('components.definitionCard.removeItemTitle')" @click.stop="$emit('remove')">
                  <Trash2 class="h-4 w-4" />
                  <span class="sr-only">{{ t('components.definitionCard.removeItemTitle') }}</span>
                </button>
              </div>
              <div v-if="showExternalDeleteButton && isOwner" class="flex flex-wrap justify-end ml-auto" role="group">
                <button class="btn-empty hover:text-red-600 flex items-center gap-1"
                  :title="t('components.definitionCard.removeFromLevelTitle')" @click.stop="$emit('delete-item')">
                  <MinusCircle class="h-4 w-4" />
                  <span>{{ t('components.definitionCard.removeFromLevelButton') }}</span>
                </button>
              </div>
            </div>


            <!-- Metadata Row -->
            <div class="flex flex-wrap items-center gap-2 mt-1 text-sm text-gray-500">
              <span v-if="definition.definitionid && definition.langid" class="italic text-gray-600">
                {{ getLanguageName(definition.langid ?? definition.lang_id) }}
              </span>
              <span v-if="definition.definitionid && definition.username">·</span>
              <span v-if="definition.username">
                {{ t('components.definitionCard.by') }}
                <RouterLink :to="`/user/${definition.username}`"
                  class="text-blue-600 hover:text-blue-800 hover:underline">
                  {{ definition.username }}
                </RouterLink>
              </span>
              <span v-if="definition.created_at && definition.username">·</span>
              <span v-if="definition.created_at">
                {{ formatDate(definition.created_at) }}
              </span>
              <span v-if="props.showScore && definition.similarity" class="italic">
                · {{ definition.similarity.toFixed(2) }} {{ t('components.definitionCard.similarity') }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Images and Content Layout -->
      <div class="mt-4 space-y-4">
        <!-- Front Image -->
        <div v-if="definition.has_front_image" class="flex justify-center">
          <img :src="`/api/collections/${collectionId}/items/${itemId}/image/front`" alt="Front side image"
            class="max-h-48 w-full rounded-lg object-contain bg-gray-50 hover:bg-gray-100 transition-colors">
        </div>
      </div>

      <!-- Definition Content -->
      <div v-if="(hasNotes || showNotesEdit) && definition.definitionid" class="mt-3 pt-2 border-t">
        <div v-if="hasNotes">
          <div class="text-sm text-gray-600 bg-gray-50 rounded">
            <LazyMathJax v-if="notes" :content="notes" :enable-markdown="true" />
          </div>
        </div>

      </div>

      <!-- Back Image -->
      <div v-if="definition.has_back_image" class="flex justify-center">
        <img :src="`/api/collections/${collectionId}/items/${definition.item_id}/image/back`" alt="Back side image"
          class="max-h-48 w-full rounded-lg object-contain bg-gray-50 hover:bg-gray-100 transition-colors">
      </div>

      <div class="text-sm prose prose-sm max-w-none text-gray-700 mt-2">
        <LazyMathJax :content="definition.definition || definition.free_content_back || definition.content" />
      </div>

      <!-- Additional Info -->
      <div v-if="hasAdditionalInfo" class="mt-2 flex flex-col gap-2">
        <div v-if="definition.notes" class="w-full text-sm text-gray-600 bg-gray-100 p-2 rounded-md">
          <h4 class="italic text-gray-600">
            {{ t('components.definitionCard.notesLabel') }}
          </h4>
          <LazyMathJax :content="definition.notes" :enable-markdown="true" />
        </div>

        <div v-if="definition.has_image" class="mt-4 flex justify-center">
          <img :src="`/api/jbovlaste/definition_image/${definition.definitionid}/image`" alt="Definition image"
            class="max-h-64 rounded-lg object-contain bg-gray-100">
        </div>

        <div v-if="definition.etymology" class="w-full text-sm text-gray-600 bg-gray-100 p-2 rounded-md">
          <h4 class="italic text-gray-600">
            {{ t('components.definitionCard.etymologyLabel') }}
          </h4>
          <LazyMathJax :content="definition.etymology" :enable-markdown="true" />
        </div>

        <div v-if="definition.examples && definition.examples.length > 0" class="w-full mt-2">
          <h4 class="italic text-gray-600 mb-2">
            {{ t('components.definitionCard.examplesLabel') }}
          </h4>
          <div class="overflow-x-auto border rounded-md">
            <table class="min-w-full divide-y divide-gray-200 text-sm">
              <thead class="bg-gray-50 uppercase tracking-wider text-[10px] font-semibold text-gray-500">
                <tr>
                  <th scope="col" class="px-3 py-2 text-left">#</th>
                  <th scope="col" class="px-3 py-2 text-left">Content</th>
                  <th scope="col" class="px-3 py-2 text-left">By</th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-100">
                <tr v-for="ex in definition.examples" :key="ex.exampleid" class="hover:bg-gray-50/50">
                  <td class="px-3 py-2 whitespace-nowrap text-gray-400 font-mono text-xs">
                    {{ ex.examplenum }}
                  </td>
                  <td class="px-3 py-2 text-gray-700">
                    <LazyMathJax :content="ex.content" :enable-markdown="true" />
                  </td>
                  <td class="px-3 py-2 whitespace-nowrap text-gray-500 text-xs">
                    {{ ex.username }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div v-if="definition.owner_only && !props.disableOwnerOnlyLock" class="flex items-center text-amber-600">
          <Lock class="h-5 w-5 mr-1" />
          <span class="text-sm">{{ t('components.definitionCard.ownerOnlyNote') }}</span>
        </div>
      </div>

      <!-- Canonical Form -->
      <div v-if="(flashcard && flashcard.canonical_form) || (definition && definition.canonical_form)"
        class="mt-3 pt-3 border-t flex flex-col gap-1.5">
        <div class="flex items-center gap-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">
          <EqualApproximately class="h-3.5 w-3.5 text-blue-400" />
          <span>{{ t('components.definitionCard.canonicalLabel') }}</span>
        </div>
        <div
          class="text-sm text-gray-700 font-mono bg-blue-50/30 p-2 rounded border border-blue-100/30 whitespace-pre-wrap leading-relaxed overflow-x-auto">
          {{ flashcard?.canonical_form || definition.canonical_form }}
        </div>
      </div>

      <!-- Control Section -->
      <div v-if="!disableToolbar" class="mt-3">
        <div class="flex flex-wrap gap-2 sm:gap-0 sm:space-x-2 md:space-x-0 space-x-0" role="group">
          <!-- Edit button -->
          <button v-if="auth.state.isLoggedIn && definition.can_edit"
            class="btn-update btn-group-item w-full sm:w-auto text-center"
            @click="router.push(`/definition/${definition.definitionid}/edit`)">
            <Pencil class="h-4 w-4" />
            {{ t('components.definitionCard.editButton') }}
          </button>

          <!-- Delete button -->
          <button v-if="auth.state.isLoggedIn && auth.state.username === definition.username" @click="handleDeleteClick"
            class="btn-delete btn-group-item w-full sm:w-auto text-center">
            <Trash2 class="h-4 w-4" />
            {{ t('components.definitionCard.deleteButton') }}
          </button>

          <!-- Version History -->
          <RouterLink v-if="auth.state.isLoggedIn && !props.hideHistory"
            :to="`/definition/${definition.definitionid}/history?valsi_id=${definition.valsiid}`"
            class="btn-history btn-group-item w-full sm:w-auto text-center">
            <Clock class="h-4 w-4" />
            {{ t('components.definitionCard.historyButton') }}
          </RouterLink>

          <!-- Comment -->
          <RouterLink v-if="auth.state.isLoggedIn && props.showCommentButton"
            :to="`/comments?valsi_id=${definition.valsiid}&definition_id=${definition.definitionid}`"
            class="btn-create btn-group-item w-full sm:w-auto text-center">
            <MessageSquare class="h-4 w-4" />
            {{ t('components.definitionCard.commentButton') }}
          </RouterLink>

          <!-- Translate -->
          <button v-if="auth.state.isLoggedIn"
            class="btn-create btn-group-item w-full sm:w-auto text-center"
            @click="router.push(`/valsi/add?word=${encodeURIComponent(definition.valsiword ?? definition.word)}${canLink ? '&translate_from_def=' + definition.definitionid : ''}`)"
            :title="canLink ? t('components.definitionCard.translateButtonTitlePhrase') : t('components.definitionCard.translateButtonTitle')">
            <Languages class="h-4 w-4" />
            {{ t('components.definitionCard.translateButton') }}
          </button>

          <!-- Link existing -->
          <button v-if="auth.state.isLoggedIn && canLink"
            class="btn-aqua-blue btn-group-item w-full sm:w-auto text-center"
            @click="showLinkModal = true"
            :title="t('components.definitionCard.linkExistingTitle')">
            <LinkIcon class="h-4 w-4" />
            {{ t('components.definitionCard.linkExisting') }}
          </button>

          <!-- Discussions -->
          <RouterLink v-if="disableDiscussionButton && !disableDiscussionToolbarButton"
            :to="`/comments?valsi_id=${definition.valsiid}&definition_id=${definition.definitionid}`"
            class="btn-get btn-group-item w-full sm:w-auto text-center">
            <AudioWaveform class="h-4 w-4" />
            <span v-if="definition.comment_count && definition.comment_count > 0"
              class="bg-gray-100 px-1.5 rounded-md border">
              {{ definition.comment_count }}
            </span>
            {{ t('components.definitionCard.discussDefinitionButton') }}
          </RouterLink>
        </div>
      </div>
    </div>
  </div>
  <ModalComponent
    :show="showValsiModal"
    :title="t('components.definitionCard.fullWordModalTitle')"
    @close="showValsiModal = false">
    <p class="text-sm text-gray-600 mb-3">{{ t('components.definitionCard.fullWordModalHint') }}</p>
    <RouterLink
      :to="valsiDefinitionLink"
      class="text-blue-700 hover:text-blue-800 hover:underline break-all font-medium"
      @click="showValsiModal = false">
      {{ definition.valsiword ?? definition.word }}
    </RouterLink>
  </ModalComponent>

  <!-- Link Existing Definition Modal -->
  <ModalComponent
    :show="showLinkModal"
    :title="t('components.definitionCard.modalTitle')"
    @close="showLinkModal = false">
    <div class="space-y-4">
      <div class="relative">
        <input
          v-model="linkSearchQuery"
          type="text"
          :placeholder="t('components.definitionCard.searchPlaceholder')"
          class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 pr-10"
          @input="handleLinkSearch"
        >
        <div class="absolute right-3 top-2.5">
          <Loader2 v-if="isSearching" class="h-5 w-5 animate-spin text-blue-500" />
          <Search v-else class="h-5 w-5 text-gray-400" />
        </div>
      </div>

      <div class="max-h-96 overflow-y-auto space-y-2">
        <div v-if="linkSearchResults.length === 0 && !isSearching" class="text-center py-4 text-gray-500">
          {{ t('components.definitionCard.noResults') }}
        </div>
        
        <div v-for="res in linkSearchResults" :key="res.definitionid" 
          class="p-3 border rounded-lg hover:border-blue-300 hover:bg-blue-50 cursor-pointer transition-colors group"
          @click="handleLinkTo(res)">
          <div class="flex justify-between items-start">
            <div class="min-w-0 flex-1">
              <div class="font-bold text-blue-700 truncate group-hover:underline">
                {{ res.valsiword }}
              </div>
              <div class="text-sm text-gray-600 line-clamp-2">
                {{ res.definition }}
              </div>
              <div class="mt-1 flex items-center gap-2 text-xs text-gray-400">
                <span>{{ t('components.definitionCard.language') }} {{ getLanguageName(res.langid) }}</span>
                <span>·</span>
                <span>{{ t('components.definitionCard.wordType') }} {{ res.type_name }}</span>
              </div>
            </div>
            <button class="ml-2 btn-aqua-emerald py-1 px-3 text-sm">
              {{ t('components.definitionCard.link') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </ModalComponent>
  <DeleteConfirmation :show="showDeleteConfirm" :title="t('components.definitionCard.deleteConfirmTitle')"
    :message="t('components.definitionCard.deleteConfirmMessage', { word: definition.valsiword ?? definition.word })"
    :is-deleting="isDeleting" @confirm="confirmDelete" @cancel="cancelDelete" />
</template>

<script setup>
import {
  ArrowUp,
  ArrowDown,
  Trash2,
  Pencil,
  Lock,
  MessageSquare,
  Clock,
  AudioWaveform,
  MessageSquarePlus,
  MessageSquareMore,
  MinusCircle,
  GalleryHorizontalIcon,
  EqualApproximately,
  Languages,
  Link as LinkIcon,
  Search,
  Loader2,
} from 'lucide-vue-next'
import { computed, ref, watch } from 'vue';
import { RouterLink, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { getTypeClass } from '@/utils/wordTypeUtils'; // Import shared utility

import { deleteDefinition, searchDefinitions, linkDefinitions } from '@/api';
import ClipboardButton from '@/components/ClipboardButton.vue';
import CollectionWidget from '@/components/CollectionWidget.vue';
import DeleteConfirmation from '@/components/DeleteConfirmation.vue';
import ModalComponent from '@/components/ModalComponent.vue';
import VoteButtons from '@/components/VoteButtons.vue';
import { useAuth } from '@/composables/useAuth';
import { useError } from '@/composables/useError';
import AudioPlayer from './AudioPlayer.vue';
import LazyMathJax from './LazyMathJax.vue';

const { t, locale } = useI18n();
const auth = useAuth();
const router = useRouter();
const { showError } = useError();

const props = defineProps({
  collections: {
    type: Array,
    default: () => [],
  },
  showDefinitionNumber: {
    type: Boolean,
  },
  definitionId: {
    type: Number,
    default: null,
  },
  showAudio: {
    type: Boolean,
    default: true,
  },
  showWordType: {
    type: Boolean,
    default: true,
  },
  showVoteButtons: {
    type: Boolean,
    default: true,
  },
  disableDiscussionButton: {
    type: Boolean,
    default: false,
  },
  disableOwnerOnlyLock: {
    type: Boolean,
    default: false,
  },
  disableDiscussionToolbarButton: {
    type: Boolean,
    default: false,
  },
  disableToolbar: {
    type: Boolean,
    default: false,
  },
  disableBorder: {
    type: Boolean,
    default: false,
  },
  definition: {
    type: Object,
    required: true,
  },
  languages: {
    type: Array,
    required: true,
  },
  notes: {
    type: String,
    default: null,
  },
  showCommentButton: {
    type: Boolean,
    default: false,
  },
  hideHistory: {
    type: Boolean,
    default: false,
  },
  collectionId: { type: Number },
  itemId: { type: Number },
  showScore: {
    type: Boolean,
    default: false,
  },
  showEditButton: {
    type: Boolean,
    default: false,
  },
  showReorderControls: {
    type: Boolean,
    default: false,
  },
  showNotesEdit: {
    type: Boolean,
    default: false,
  },
  flashcard: {
    type: Object,
    default: null,
  },
  isOwner: {
    type: Boolean,
    default: false,
  },
  isReordering: {
    type: Boolean,
    default: false,
  },
  isFirstItem: {
    type: Boolean,
    default: false,
  },
  isLastItem: {
    type: Boolean,
    default: false,
  },
  showExternalDeleteButton: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['move-up', 'move-down', 'remove', 'collection-updated', 'delete', 'refresh-definitions', 'edit-item', 'delete-item'])

const MAX_VALSI_DISPLAY_LENGTH = 30

const collections = ref(props.collections)
const showDeleteConfirm = ref(false)
const showValsiModal = ref(false)
const isDeleting = ref(false)

const valsiWord = computed(() => props.definition.valsiword ?? props.definition.word)
const displayedValsi = computed(() =>
  valsiWord.value.length > MAX_VALSI_DISPLAY_LENGTH
    ? valsiWord.value.slice(0, MAX_VALSI_DISPLAY_LENGTH) + '…'
    : valsiWord.value
)
const isValsiTruncated = computed(() => valsiWord.value.length > MAX_VALSI_DISPLAY_LENGTH)
const valsiDefinitionLink = computed(() =>
  props.definition.definitionid
    ? `/valsi/${encodeURIComponent(valsiWord.value.replace(/ /g, '_'))}?highlight_definition_id=${props.definition.definitionid}`
    : '#'
)
const displayedFreeContent = computed(() => {
  const raw = props.definition.free_content_front || props.definition.word || ''
  return raw.length > MAX_VALSI_DISPLAY_LENGTH ? raw.slice(0, MAX_VALSI_DISPLAY_LENGTH) + '…' : raw
})
const displayedSelmaho = computed(() => {
  const s = props.definition.selmaho || ''
  return s.length > MAX_VALSI_DISPLAY_LENGTH ? s.slice(0, MAX_VALSI_DISPLAY_LENGTH) + '…' : s
})

const canLink = computed(() => props.definition.type_name === 'phrase')

// Linking Modal State
const showLinkModal = ref(false)
const linkSearchQuery = ref('')
const linkSearchResults = ref([])
const isSearching = ref(false)
let searchTimeout = null

const handleLinkSearch = () => {
  if (searchTimeout) clearTimeout(searchTimeout)
  if (!linkSearchQuery.value.trim()) {
    linkSearchResults.value = []
    return
  }
  
  searchTimeout = setTimeout(async () => {
    isSearching.value = true
    try {
      const res = await searchDefinitions({
        query: linkSearchQuery.value,
        word_type: 'phrase',
        per_page: 20
      })
      // Filter out current definition
      linkSearchResults.value = res.data.definitions.filter(d => d.definitionid !== props.definition.definitionid)
    } catch (e) {
      console.error('Search error', e)
    } finally {
      isSearching.value = false
    }
  }, 400)
}

const handleLinkTo = async (target) => {
  if (!confirm(t('components.definitionCard.linkConfirm'))) return
  
  try {
    await linkDefinitions(props.definition.definitionid, target.definitionid)
    showLinkModal.value = false
    emit('refresh-definitions')
  } catch (e) {
    console.error('Linking error', e)
    showError(e.response?.data?.error || t('components.definitionCard.linkError'))
  }
}

watch(
  () => props.collections,
  (newCollections) => {
    collections.value = newCollections
  },
  { immediate: true }
)

const updateCollections = (updatedCollections) => {
  collections.value = updatedCollections
  emit('collection-updated', updatedCollections)
}

const hasNotes = computed(() =>
  Boolean(
    (props.notes || '').trim() !== '' ||
    props.definition.has_front_image ||
    props.definition.has_back_image
  )
)

const hasAdditionalInfo = computed(() => {
  return props.definition.notes || props.definition.selmaho
})

const selmahoLinkQuery = computed(() => {
  return {
    mode: 'dictionary',
    selmaho: props.definition.selmaho,
    q: '',
  }
})

// Remove the local getTypeClass implementation

const getLanguageName = (langId) => {
  const lang = props.languages.find((l) => l.id === langId)
  return lang ? lang.real_name : ''
}

const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
  })
}

const handleDeleteClick = () => {
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  if (!props.definition.definitionid) return
  isDeleting.value = true
  try {
    const valsiWord = props.definition.valsiword ?? props.definition.word
    const response = await deleteDefinition(props.definition.definitionid)
    emit('refresh-definitions') // Notify parent to refresh the list
    showDeleteConfirm.value = false

    // Check if valsi was deleted or if there are no remaining definitions
    const responseData = response.data
    if (responseData.valsi_deleted || !responseData.has_remaining_definitions) {
      // Redirect to home page if valsi was deleted or no definitions remain
      router.push('/')
    } else {
      // Redirect to valsi page if definitions still exist
      router.push(`/valsi/${valsiWord}`)
    }
  } catch (error) {
    console.error('Error deleting definition:', error)
    // Optionally show an error message to the user
  } finally {
    isDeleting.value = false
  }
}

const cancelDelete = () => {
  showDeleteConfirm.value = false
}
</script>
