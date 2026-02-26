<template>
  <div v-if="collection">
    <ToastFloat :show="showSuccessToast" :message="successMessage" type="success" @close="showSuccessToast = false" />
    <!-- Header -->
    <div class="bg-white border rounded-lg p-4 sm:p-6 mb-6">
      <div class="flex flex-col sm:flex-row justify-between items-start gap-2 sm:gap-0">
        <div class="w-full sm:w-auto">
          <h2 class="text-xl sm:text-2xl font-bold text-gray-800 flex items-center gap-2">
            {{ collection.name }}
            <span class="text-sm px-2 py-1 rounded-full select-none" :class="collection.is_public ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'
              ">{{ collection.is_public ? t('collectionDetail.public') : t('collectionDetail.private') }}</span>
          </h2>

          <!-- Meta information -->
          <div class="flex flex-wrap gap-2 mt-4">
            <div class="text-sm text-gray-500">
              {{ t('collectionDetail.createdBy') }}
              <RouterLink :to="`/user/${collection.owner.username}`"
                class="text-blue-600 hover:text-blue-800 hover:underline">
                {{ collection.owner.username }}
              </RouterLink>
            </div>
            <div class="text-sm text-gray-500">
              {{ t('collectionDetail.itemsCount', { count: collection.item_count }) }}
            </div>
          </div>
        </div>

        <!-- Actions Dropdown -->
        <div v-if="auth.state.isLoggedIn" class="relative actions-dropdown w-full sm:w-auto">
          <button
            class="w-full sm:w-auto p-2 hover:bg-gray-100 rounded-full inline-flex items-center justify-between sm:justify-start gap-2"
            @click="showActions = !showActions">
            <span class="text-sm text-gray-600">{{ t('collectionDetail.actions') }}</span>
            <EllipsisVertical class="w-4 h-4" />
          </button>
          <div v-if="showActions"
            class="absolute right-0 mt-2 w-full sm:w-48 bg-white border rounded-lg shadow-lg py-1 z-30">
            <button v-if="isOwner" class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100"
              @click="showEditModal = true">
              {{ t('collectionDetail.editCollectionInfo') }}
            </button>
            <button class="w-full px-4 py-2 text-left text-sm text-indigo-600 hover:bg-indigo-50"
              @click="handleCloneCollection">
              {{ t('collectionDetail.cloneCollection') }}
            </button>
            <button v-if="collection.is_public || isOwner"
              class="w-full px-4 py-2 text-left text-sm text-purple-600 hover:bg-purple-50"
              @click="showExportModal = true">
              {{ t('collectionDetail.exportCollection') }}
            </button>
            <button v-if="isOwner" class="w-full px-4 py-2 text-left text-sm text-green-600 hover:bg-green-50"
              @click="showMergeModal = true, loadAvailableCollections()">
              {{ t('collectionDetail.mergeCollections') }}
            </button>
            <button v-if="isOwner" class="w-full px-4 py-2 text-left text-sm text-cyan-600 hover:bg-cyan-50"
              @click="triggerJsonImport">
              {{ t('collectionDetail.importFullButton', 'Import from file') }}
            </button>
            <button v-if="isOwner" class="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50"
              @click="handleDelete">
              {{ t('collectionDetail.deleteCollection') }}
            </button>
          </div>
        </div>
      </div>


      <div v-if="collection.description">
        <div class="max-h-32 text-sm overflow-y-auto border rounded p-2 bg-gray-50 my-2 w-full read-box">
          <LazyMathJax :content="collection.description" />
        </div>
      </div>

      <!-- Hidden file input for JSON import -->
      <input ref="jsonImportInput" type="file" accept=".json" class="hidden" @change="handleJsonFileSelect">


      <!-- Search and Actions -->
      <div class="space-y-4 md:space-y-0">
        <div class="flex flex-wrap items-center gap-2 w-auto">
          <div class="relative w-full md:w-auto mb-2 md:mb-0">
            <SearchInput v-model="itemSearchQuery" :placeholder="t('collectionDetail.searchItemsPlaceholder')"
              :is-loading="isSearching" @update:model-value="handleSearch" @clear="clearItemSearch" />
          </div>
          <button v-if="isOwner" class="btn-aqua-green md:flex-none" @click="resetForm(); showAddModal = true">
            <PlusCircle class="w-4 h-4" />
            {{ t('collectionDetail.addItem') }}
          </button>
          <RouterLink :to="`/collections/${props.collectionId}/flashcards`" class="btn-aqua-orange md:flex-none">
            <GalleryHorizontalIcon class="w-4 h-4" />
            {{ t('collectionDetail.viewAsFlashcards') }}
          </RouterLink>
          <RouterLink :to="`/collections/${props.collectionId}/levels`" class="btn-aqua-sky ymd:flex-none">
            <LayoutPanelTop class="w-4 h-4" />
            {{ t('collectionDetail.levels') }}
          </RouterLink>
        </div>
      </div>
    </div>
    <!-- Loading State -->
    <LoadingSpinner v-if="isLoading" class="py-12" />
    <!-- Items List -->
    <div v-if="isOwner && isAddFlashcardMode" class="flex items-center gap-2 px-3 py-2 my-2">
      <label class="btn-aqua-slate">
        <input type="checkbox" class="checkmark-aqua" :checked="isAddFlashcardMode" @change="toggleAddFlashcardMode">
        <span> {{ t('collectionDetail.onlyItemsWithoutFlashcards') }} </span>
      </label>
    </div>
    <div class="space-y-4">
      <!-- Loading state for items -->
      <LoadingSpinner v-if="isLoadingItems" class="py-8" />

      <!-- Items grid -->
      <div v-else class="grid gap-4">
        <div v-for="(item, index) in paginatedItems.items" :key="item.item_id"
          class="relative w-full max-w-full overflow-visible">
          <DefinitionCard :show-edit-button="isOwner" :show-reorder-controls="isOwner" :is-owner="isOwner"
            :flashcard="item.flashcard" :is-reordering="isReordering" :show-vote-buttons="false"
            :is-first-item="index === 0" :is-last-item="index === paginatedItems.items.length - 1"             :definition="{
              definitionid: item.definition_id || 0,
              item_id: item.item_id,
              valsiid: item.valsi_id,
              valsiword: item.word ?? item.free_content_front,
              definition: item.definition ?? item.free_content_back,
              langid: item.lang_id,
              notes: item.notes,
              free_content_front: item.free_content_front,
              free_content_back: item.free_content_back,
              has_front_image: item.has_front_image,
              has_back_image: item.has_back_image,
              has_sound: item.has_sound,
              sound_url: item.sound_url,
            }" :collection-id="collection.collection_id" :item-id="item.item_id" :languages="languages"
            :collections="userCollections" @collection-updated="userCollections = $event"
            :disable-discussion-button="true" :disable-toolbar="true" :notes="item.ci_notes" :show-notes-edit="isOwner"
            @move-up="moveItem(item, 'up')" @move-down="moveItem(item, 'down')" @remove="confirmRemoveItem(item)"
            @edit-item="openEditItemModal(item)" />
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="!isLoadingItems && paginatedItems.items.length === 0"
        class="text-center py-12 bg-gray-50 rounded-lg border border-gray-200">
        <p class="text-gray-600">
          {{ t('collectionDetail.noItems') }}
        </p>
        <div class="flex justify-center">
          <button v-if="isOwner" class="mt-4 btn-aqua-emerald" @click="showAddModal = true">
            {{ t('collectionDetail.addItemButton') }}
          </button>
        </div>
      </div>

      <!-- PaginationComponent -->
      <PaginationComponent v-if="paginatedItems.total > itemsPerPage" :current-page="currentPage"
        :total-pages="totalPages" :total="paginatedItems.total" :per-page="itemsPerPage" @prev="prevPage"
        @next="nextPage" />
    </div>
    <!-- Edit Collection Modal -->
    <ModalComponent :show="showEditModal" :title="t('collectionDetail.editCollectionTitle')"
      @close="cancelEditCollectionModal()">
      <form @submit.prevent="performUpdateCollection">
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.nameLabel') }}</label>
            <input v-model="editForm.name" type="text" required class="input-field w-full">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.descriptionLabel')
            }}</label>
            <textarea v-model="editForm.description" rows="3" class="textarea-field" />
          </div>
          <div class="flex items-center">
            <input id="edit_is_public" v-model="editForm.is_public" type="checkbox" class="checkbox-toggle">
            <label for="edit_is_public" class="ml-2 text-sm text-gray-700">
              {{ t('collectionDetail.makePublicLabel') }}
            </label>
          </div>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <button type="button" class="btn-cancel" @click="cancelEditCollectionModal()">
            {{ t('collectionDetail.cancel') }}
          </button>
          <button type="submit" :disabled="isSubmitting" class="btn-update">
            {{ isSubmitting ? t('collectionDetail.saving') : t('collectionDetail.saveChanges') }}
          </button>
        </div>
      </form>
    </ModalComponent>

    <!-- Export Collection Modal (full: items + levels + flashcard directions) -->
    <ModalComponent :show="showExportModal" :title="t('collectionDetail.exportCollectionTitle')"
      @close="showExportModal = false; showActions = !showActions">
      <div class="space-y-4">
        <p class="text-sm text-gray-600">
          {{ t('collectionDetail.exportFullDescription', 'Exports this collection as JSON including all items, flashcard levels, and flashcard settings. You can re-import the file later to create a new collection with the same structure.') }}
        </p>
        <div v-if="exportProgress" class="text-sm text-blue-600 mt-2">
          {{ exportProgress }}
        </div>
        <div v-if="exportError" class="text-sm text-red-600 mt-2">
          {{ exportError }}
        </div>
      </div>

      <div class="mt-6 flex justify-end space-x-3">
        <button type="button" class="btn-cancel" @click="showExportModal = false; showActions = !showActions">
          {{ t('collectionDetail.cancel') }}
        </button>
        <button :disabled="isExporting" class="btn-get" @click="handleExport">
          {{ isExporting ? t('collectionDetail.exporting') : t('collectionDetail.exportButton') }}
        </button>
      </div>
    </ModalComponent>

    <!-- Add/Edit Item Modal -->
    <ModalComponent :show="showAddModal"
      :title="isEditingItem ? t('collectionDetail.editItemTitle') : t('collectionDetail.addItemTitle')"
      @close="cancelEditItemModal()">
      <!-- Item Type Selection -->
      <TabbedPageHeader :tabs="addItemTabs" :active-tab="itemType"
        :page-title="addItemTabs.find(t => t.key === itemType)?.label || ''" @tab-click="itemType = $event" />

      <!-- Custom Content Form -->
      <div v-if="itemType === 'custom'" class="flex-1 overflow-y-auto space-y-2 p-2" ref="customContentContainer">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.frontContentLabel')
          }}</label>
          <textarea v-model="customContent.front" rows="3" :placeholder="t('collectionDetail.frontContentPlaceholder')"
            class="textarea-field" :class="{ 'border-red-300': showValidation && !customContent.front.trim() }" />
          <span v-if="showValidation && !customContent.front.trim()" class="text-sm text-red-600">{{
            t('collectionDetail.frontContentRequired') }}</span>
          <p class="text-xs text-gray-500 mt-1">{{ t('collectionDetail.semicolonHint') }}</p>
        </div>

        <ImageUpload v-model="customContent.frontImage" :collection-id="numericCollectionId" :item-id="null"
          side="front" :label="t('collectionDetail.frontImageLabel')" @image-loaded="handleFrontImageLoaded" />

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.backContentLabel')
          }}</label>
          <textarea v-model="customContent.back" rows="3" :placeholder="t('collectionDetail.backContentPlaceholder')"
            class="textarea-field" :class="{ 'border-red-300': showValidation && !customContent.back.trim() }" />
          <span v-if="showValidation && !customContent.back.trim()" class="text-sm text-red-600">{{
            t('collectionDetail.backContentRequired') }}</span>
          <p class="text-xs text-gray-500 mt-1">{{ t('collectionDetail.semicolonHint') }}</p>
        </div>

        <ImageUpload v-model="customContent.backImage" :collection-id="numericCollectionId" :item-id="null" side="back"
          :label="t('collectionDetail.backImageLabel')" @image-loaded="handleBackImageLoaded" />

        <SoundUpload v-model="customContent.sound" :collection-id="numericCollectionId" :item-id="currentItem?.item_id"
          :has-existing-sound="currentItem?.has_sound" :label="t('collectionDetail.soundLabel', 'Custom Sound')"
          @remove-sound="customContent.removeSound = true" />
        <!-- Notes Field -->
        <div class="mt-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.notesLabel') }}</label>
          <textarea v-model="addItemNotes" rows="2" class="textarea-field"
            :placeholder="t('collectionDetail.notesPlaceholder')" />
        </div>
        <div class="flex items-center space-x-2 mb-2">
          <button type="button" class="btn-action" @click="toggleFlashcard">
            <template v-if="enableFlashcard">
              {{ t('collectionDetail.removeFlashcard') }}
            </template>
            <template v-else>
              {{ t('collectionDetail.createFlashcard') }}
            </template>
          </button>
        </div>

        <div v-if="enableFlashcard" class="space-y-2">
          <label v-if="itemType !== 'quiz'" class="block text-sm font-medium text-gray-700 mb-2">{{ t('collectionDetail.studyDirectionLabel')
          }}</label>
          <select v-if="itemType !== 'quiz'" v-model="addItemDirection" class="w-full input-field text-sm">
            <option value="direct">{{ t('collectionDetail.direction.direct') }}</option>
            <option value="reverse">{{ t('collectionDetail.direction.reverse') }}</option>
            <option value="both">{{ t('collectionDetail.direction.both') }}</option>
            <option value="fillin">{{ t('collectionDetail.direction.fillin') }}</option>
            <option value="fillin_reverse">{{ t('collectionDetail.direction.fillin_reverse') }}</option>
            <option value="fillin_both">{{ t('collectionDetail.direction.fillin_both') }}</option>
            <option value="justinformation">{{ t('collectionDetail.direction.justinformation') }}</option>
          </select>
          <label v-if="itemType === 'quiz'" class="block text-sm font-medium text-gray-700 mb-2">{{ t('collectionDetail.quizDirectionLabel') }}</label>
          <select v-if="itemType === 'quiz'" v-model="addItemDirection" class="w-full input-field text-sm">
            <option value="QuizDirect">{{ t('collectionDetail.direction.direct') }}</option>
            <option value="QuizReverse">{{ t('collectionDetail.direction.reverse') }}</option>
            <option value="QuizBoth">{{ t('collectionDetail.direction.both') }}</option>
          </select>

          <div class="flex items-center space-x-2">
            <input id="auto_progress" v-model="customContent.auto_progress" type="checkbox" class="checkbox-toggle">
            <label for="auto_progress" class="text-sm text-gray-700"> {{ t('collectionDetail.autoProgressLabel') }}
            </label>
          </div>

          <p class="text-xs text-gray-500">
            {{ t('collectionDetail.autoProgressDescription') }}
          </p>

          <!-- Canonical Comparison Checkbox for Fill-in Mode -->
          <div v-if="addItemDirection.toLowerCase().includes('fillin')" class="flex items-center space-x-2 mt-3">
            <input id="use_canonical_comparison" v-model="useCanonicalComparison" type="checkbox" class="checkbox-toggle">
            <label for="use_canonical_comparison" class="text-sm text-gray-700">
              {{ t('collectionDetail.useCanonicalComparisonLabel', 'Use canonical Lojban comparison') }}
            </label>
          </div>
          <p v-if="addItemDirection.toLowerCase().includes('fillin')" class="text-xs text-gray-500">
            {{ t('collectionDetail.useCanonicalComparisonDescription', 'When enabled, answers will be compared using their canonical Lojban form, allowing for equivalent but differently written answers.') }}
          </p>
        </div>

      </div>

      <!-- Definition Search -->
      <div v-else-if="itemType === 'definition'" class="flex-1 overflow-y-auto space-y-2 p-2"
        ref="searchContentContainer">
        <div class="mb-4">
          <SearchInput v-model="searchQuery" :placeholder="t('collectionDetail.searchDefinitionsPlaceholder')"
            :is-loading="isSearching" class="w-full text-base h-10" @update:model-value="debouncedSearch"
            @clear="clearDefinitionSearch" />
        </div>
        <!-- Search Results -->
        <div class="flex-1 pr-2" :class="{ 'overflow-y-auto': !selectedDefinition }">
          <div v-if="!searchQuery" class="text-center py-8 text-gray-600">
            {{ t('collectionDetail.searchPrompt') }}
          </div>
          <div v-else-if="addItemResults.length === 0" class="text-center py-8 text-gray-600">
            {{ t('collectionDetail.noDefinitionsFound') }}
          </div>
          <div v-else-if="!!selectedDefinition">
            <div class="border rounded-lg p-4 hover:border-blue-300 cursor-pointer border-blue-500 bg-blue-50"
              :data-definition-id="selectedDefinition.definitionid" @click="selectDefinition(selectedDefinition)">
              <div>
                <div class="flex justify-between items-center w-full">
                  <h4 class="font-medium text-blue-600">
                    {{ selectedDefinition.valsiword || selectedDefinition.free_content_front }}
                  </h4>
                  <button class="btn-empty">
                    {{ t('collectionDetail.deselect') }}
                  </button>
                </div>

                <div v-if="selectedDefinition.username" class="mt-2 text-sm text-gray-500">
                  {{ t('collectionDetail.addedByIn', {
                    username: selectedDefinition.username, language:
                      selectedDefinition.langrealname
                  }) }}
                </div>

                <LazyMathJax :content="selectedDefinition.definition || selectedDefinition.free_content_back" />
              </div>
            </div>
          </div>
          <div v-else class="space-y-4">
            <div v-for="def in addItemResults" :key="def.definitionid"
              class="border rounded-lg p-4 hover:border-blue-300 cursor-pointer" :data-definition-id="def.definitionid"
              @click="selectDefinition(def)">
              <div>
                <div class="flex justify-between items-center w-full mb-2">
                  <h4 class="font-medium text-blue-600">
                    {{ def.valsiword || def.free_content_front }}
                  </h4>
                  <button class="btn-empty">
                    {{ t('collectionDetail.select') }}
                  </button>
                </div>

                <div v-if="def.username" class="mt-2 text-sm text-gray-500">
                  {{ t('collectionDetail.addedByIn', { username: def.username, language: def.langrealname }) }}
                </div>

                <LazyMathJax :content="def.definition || def.free_content_back" />
              </div>
            </div>
          </div>
        </div>
        <!-- Notes Field -->
        <div v-if="selectedDefinition" class="space-y-2">
          <div class="mt-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.notesLabel') }}</label>
            <textarea v-model="addItemNotes" rows="2" class="textarea-field"
              :placeholder="t('collectionDetail.notesPlaceholder')" />
          </div>

          <!-- Image Upload -->
          <ImageUpload v-model="definitionBackImage" :collection-id="numericCollectionId" :item-id="null" side="back"
            :label="t('collectionDetail.backImageLabel')" class="mb-4"
            @image-loaded="handleDefinitionBackImageLoaded" />

          <SoundUpload v-model="definitionSound" :collection-id="numericCollectionId" :item-id="currentItem?.item_id"
            :has-existing-sound="currentItem?.has_sound" :label="t('collectionDetail.soundLabel', 'Custom Sound')"
            @remove-sound="definitionRemoveSound = true" />
          <div class="flex items-center space-x-2 mb-2">
            <button type="button" class="btn-action" @click="toggleFlashcard">
              <template v-if="enableFlashcard">
                {{ t('collectionDetail.removeFlashcard') }}
              </template>
              <template v-else>
                {{ t('collectionDetail.createFlashcard') }}
              </template>
            </button>
          </div>

          <div v-if="enableFlashcard">
            <label class="block text-sm font-medium text-gray-700">{{ t('collectionDetail.studyDirectionLabel')
            }}</label>
            <select v-model="addItemDirection" class="w-full input-field text-sm">
              <option value="direct">{{ t('collectionDetail.direction.direct') }}</option>
              <option value="reverse">{{ t('collectionDetail.direction.reverse') }}</option>
              <option value="both">{{ t('collectionDetail.direction.both') }}</option>
              <option value="fillin">{{ t('collectionDetail.direction.fillin') }}</option>
              <option value="fillin_reverse">{{ t('collectionDetail.direction.fillin_reverse') }}</option>
              <option value="fillin_both">{{ t('collectionDetail.direction.fillin_both') }}</option>
              <option value="justinformation">{{ t('collectionDetail.direction.justinformation') }}</option>
            </select>
          </div>
          <div v-if="enableFlashcard" class="flex items-center space-x-2">
            <input id="definition_auto_progress" v-model="customContent.auto_progress" type="checkbox"
              class="checkbox-toggle">
            <label for="definition_auto_progress" class="text-sm text-gray-700">
              {{ t('collectionDetail.autoProgressLabel') }}
            </label>
          </div>
          <p v-if="enableFlashcard" class="text-xs text-gray-500">
            {{ t('collectionDetail.autoProgressDescription') }}
          </p>

          <!-- Canonical Comparison for Definition Fill-in -->
          <div v-if="enableFlashcard && addItemDirection.toLowerCase().includes('fillin')" class="flex items-center space-x-2 mt-3">
            <input id="definition_canonical_comparison" v-model="useCanonicalComparison" type="checkbox" class="checkbox-toggle">
            <label for="definition_canonical_comparison" class="text-sm text-gray-700">
              {{ t('collectionDetail.useCanonicalComparisonLabel', 'Use canonical Lojban comparison') }}
            </label>
          </div>
          <p v-if="enableFlashcard && addItemDirection.toLowerCase().includes('fillin')" class="text-xs text-gray-500">
            {{ t('collectionDetail.useCanonicalComparisonDescription', 'When enabled, answers will be compared using their canonical Lojban form, allowing for equivalent but differently written answers.') }}
          </p>
        </div>
      </div>

      <!-- Quiz Content Form -->
      <div v-else-if="itemType === 'quiz'" class="flex-1 overflow-y-auto space-y-2 p-2" ref="quizContentContainer">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.frontContentLabel', 'Question') }}</label>
          <textarea v-model="customContent.front" rows="3" :placeholder="t('collectionDetail.frontContentPlaceholder', 'Enter question...')"
            class="textarea-field" :class="{ 'border-red-300': showValidation && !customContent.front.trim() }" />
          <span v-if="showValidation && !customContent.front.trim()" class="text-sm text-red-600">{{ t('collectionDetail.frontContentRequired', 'Question is required') }}</span>
        </div>
        <ImageUpload v-model="customContent.frontImage" :collection-id="numericCollectionId" :item-id="null"
          side="front" :label="t('collectionDetail.frontImageLabel', 'Question Image (Optional)')" @image-loaded="handleFrontImageLoaded" />

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.backContentLabel', 'Correct Answer') }}</label>
          <textarea v-model="customContent.back" rows="3" :placeholder="t('collectionDetail.backContentPlaceholder', 'Enter correct answer...')"
            class="textarea-field" :class="{ 'border-red-300': showValidation && !customContent.back.trim() }" />
          <span v-if="showValidation && !customContent.back.trim()" class="text-sm text-red-600">{{ t('collectionDetail.backContentRequired', 'Correct answer is required') }}</span>
        </div>
        <ImageUpload v-model="customContent.backImage" :collection-id="numericCollectionId" :item-id="null" side="back"
          :label="t('collectionDetail.backImageLabel', 'Answer Image (Optional)')" @image-loaded="handleBackImageLoaded" />

        <div class="mt-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.notesLabel') }}</label>
          <textarea v-model="addItemNotes" rows="2" class="textarea-field" :placeholder="t('collectionDetail.notesPlaceholder')" />
        </div>

        <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('collectionDetail.quizDirectionLabel') }}</label>
        <select v-model="addItemDirection" class="w-full input-field text-sm">
            <option value="QuizDirect">{{ t('collectionDetail.direction.direct') }}</option>
            <option value="QuizReverse">{{ t('collectionDetail.direction.reverse') }}</option>
            <option value="QuizBoth">{{ t('collectionDetail.direction.both') }}</option>
        </select>
      </div>

      <!-- New Definition Form -->
      <div v-else-if="itemType === 'newDefinition'" class="flex-1 overflow-y-auto space-y-4 p-2"
        ref="newDefinitionContainer">
        <!-- Word Input and Analysis -->
        <div>
          <label for="new-word" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.wordLabel') }}
            <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
          <div class="flex flex-col sm:flex-row gap-2 sm:space-x-2">
            <div class="flex-1 w-full">
              <DynamicInput id="new-word" v-model="newDefinitionData.word" :is-analyzing="isAnalyzingNewWord"
                :is-submitting="isSubmittingNewDefinition" @clear-analysis="clearNewDefinitionAnalysis" />
            </div>
            <div class="flex items-center justify-end">
              <button type="button" class="w-auto h-8 btn-aqua-orange text-base"
                :disabled="isAnalyzingNewWord || isSubmittingNewDefinition || newDefinitionData.word === ''"
                @click="doAnalyzeNewWord">
                <div class="flex items-center gap-2">
                  <Loader v-if="isAnalyzingNewWord" class="h-4 w-4 animate-spin" />
                  <SearchIcon v-else class="h-4 w-4" />
                  <span>{{ t('upsertDefinition.analyzeButton') }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>

        <!-- Word Type Display -->
        <div v-if="newDefinitionWordType" class="space-y-4">
          <AlertComponent type="info" :label="t('upsertDefinition.detectedTypeLabel')">
            <p class="font-semibold">{{ newDefinitionWordType }}</p>
          </AlertComponent>
          <AlertComponent v-if="newDefinitionRecommended" type="tip"
            :label="t('upsertDefinition.recommendedWordLabel')">
            <div class="flex items-center gap-2 justify-start">
              <h2 class="font-semibold truncate">{{ newDefinitionRecommended }}</h2>
              <button type="button" class="btn-update" @click="useNewDefinitionRecommended">
                <ArrowRight class="h-4 w-4" /> {{ t('upsertDefinition.useThisButton') }}
              </button>
            </div>
          </AlertComponent>
          <div v-if="newDefinitionProblems && Object.keys(newDefinitionProblems).length > 0" class="space-y-4">
            <div v-for="(issues, category) in newDefinitionProblems" :key="category">
              <AlertComponent v-if="issues.length > 0" type="error"
                :label="category === 'regular' ? t('upsertDefinition.similarRegularGismu') : t('upsertDefinition.similarExperimentalGismu')">
                <ul class="list-disc list-inside space-y-1">
                  <li v-for="(problem, index) in issues" :key="index" class="font-semibold truncate">{{ problem }}</li>
                </ul>
              </AlertComponent>
            </div>
          </div>
        </div>

        <!-- Language Selection -->
        <div>
          <label for="new-language" class="block text-sm font-medium text-blue-700">{{
            t('upsertDefinition.languageLabel') }}
            <span class="text-red-500">{{ t('upsertDefinition.required') }}</span></label>
          <select id="new-language" v-model="newDefinitionData.langId" required class="input-field w-full h-10"
            :disabled="isLoading || isSubmittingNewDefinition">
            <option value="">{{ t('upsertDefinition.selectLanguagePlaceholder') }}</option>
            <option v-for="lang in languages" :key="lang.id" :value="lang.id">{{ lang.real_name }} ({{ lang.english_name
              }})
            </option>
          </select>
        </div>

        <!-- Definition Input -->
        <div>
          <div class="flex items-center justify-between">
            <label for="new-definition" class="block text-sm font-medium text-blue-700">{{
              t('upsertDefinition.definitionLabel') }} <span class="text-red-500">{{ t('upsertDefinition.required')
                }}</span></label>
            <span class="text-xs text-gray-500">{{ t('upsertDefinition.requiredUnlessImage') }}</span>
          </div>
          <textarea id="new-definition" v-model="newDefinitionData.definition" :required="!newDefinitionImage" rows="4"
            :class="{ 'textarea-field': true, 'border-red-300 focus:ring-red-500 focus:border-red-500': newDefinitionError, 'border-blue-300 focus:ring-blue-500 focus:border-blue-500': !newDefinitionError }"
            :disabled="isSubmittingNewDefinition" />
          <p v-if="newDefinitionError" class="mt-2 text-xs sm:text-sm text-red-600">{{ newDefinitionError }}</p>
          <p v-else class="mt-2 text-xs sm:text-sm text-gray-500">{{ t('upsertDefinition.mathjaxNote') }}</p>
        </div>

        <!-- Image Upload -->
        <ImageUpload v-model="newDefinitionImage" :label="t('collectionDetail.imageLabel')"
          @image-loaded="handleNewDefinitionImageLoaded" @remove-image="handleNewDefinitionRemoveImage" />

        <SoundUpload v-model="newDefinitionSound" :label="t('collectionDetail.soundLabel', 'Custom Sound')" />

        <!-- Notes Input -->
        <div>
          <label for="new-notes" class="block text-sm font-medium text-blue-700">{{ t('upsertDefinition.notesLabel') }}
            <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span></label>
          <textarea id="new-notes" v-model="newDefinitionData.notes" rows="3" class="textarea-field"
            :disabled="isSubmittingNewDefinition" />
        </div>

        <!-- Etymology Input -->
        <div>
          <label for="new-etymology" class="block text-sm font-medium text-blue-700">{{
            t('upsertDefinition.etymologyLabel')
            }} <span class="text-gray-500 font-normal">{{ t('upsertDefinition.optional') }}</span></label>
          <textarea id="new-etymology" v-model="newDefinitionData.etymology" rows="3" class="textarea-field"
            :disabled="isSubmittingNewDefinition" />
        </div>

        <!-- Owner Only Checkbox -->
        <div class="mb-4">
          <label class="flex items-center space-x-2">
            <input v-model="newDefinitionData.ownerOnly" type="checkbox" class="checkbox-toggle">
            <span class="text-xs sm:text-sm text-gray-700">{{ t('upsertDefinition.ownerOnlyLabel') }} <span
                class="text-gray-500">{{ t('upsertDefinition.optional') }}</span></span>
          </label>
          <p class="mt-1 text-xs sm:text-sm text-gray-500">{{ t('upsertDefinition.ownerOnlyNote') }}</p>
        </div>

        <!-- Flashcard Settings for New Definition -->
        <div class="mt-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.notesLabel') }}</label>
          <textarea v-model="addItemNotes" rows="2" class="textarea-field"
            :placeholder="t('collectionDetail.notesPlaceholder')" />
        </div>
        <div class="flex items-center space-x-2 mb-2">
          <button type="button" class="btn-action" @click="toggleFlashcard">
            <template v-if="enableFlashcard">
              {{ t('collectionDetail.removeFlashcard') }}
            </template>
            <template v-else>
              {{ t('collectionDetail.createFlashcard') }}
            </template>
          </button>
        </div>
        <div v-if="enableFlashcard" class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('collectionDetail.studyDirectionLabel')
            }}</label>
          <select v-model="addItemDirection" class="w-full input-field text-sm">
            <option value="direct">{{ t('collectionDetail.direction.direct') }}</option>
            <option value="reverse">{{ t('collectionDetail.direction.reverse') }}</option>
            <option value="both">{{ t('collectionDetail.direction.both') }}</option>
            <option value="fillin">{{ t('collectionDetail.direction.fillin') }}</option>
            <option value="fillin_reverse">{{ t('collectionDetail.direction.fillin_reverse') }}</option>
            <option value="fillin_both">{{ t('collectionDetail.direction.fillin_both') }}</option>
            <option value="justinformation">{{ t('collectionDetail.direction.justinformation') }}</option>
          </select>
          <div class="flex items-center space-x-2">
            <input id="new_def_auto_progress" v-model="customContent.auto_progress" type="checkbox"
              class="checkbox-toggle">
            <label for="new_def_auto_progress" class="text-sm text-gray-700"> {{ t('collectionDetail.autoProgressLabel')
              }}
            </label>
          </div>
          <p class="text-xs text-gray-500">{{ t('collectionDetail.autoProgressDescription') }}</p>

          <!-- Canonical Comparison for New Definition Fill-in -->
          <div v-if="addItemDirection.toLowerCase().includes('fillin')" class="flex items-center space-x-2 mt-3">
            <input id="new_def_canonical_comparison" v-model="useCanonicalComparison" type="checkbox" class="checkbox-toggle">
            <label for="new_def_canonical_comparison" class="text-sm text-gray-700">
              {{ t('collectionDetail.useCanonicalComparisonLabel', 'Use canonical Lojban comparison') }}
            </label>
          </div>
          <p v-if="addItemDirection.toLowerCase().includes('fillin')" class="text-xs text-gray-500">
            {{ t('collectionDetail.useCanonicalComparisonDescription', 'When enabled, answers will be compared using their canonical Lojban form, allowing for equivalent but differently written answers.') }}
          </p>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="mt-4 flex flex-col gap-2">
        <!-- Progress bar when updating item -->
        <div v-if="isEditingItem && isUpdatingItem" class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
          <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
        </div>
        <div class="flex justify-end gap-2">
          <button v-if="isEditingItem" class="btn-error mr-auto"
            :disabled="isUpdatingItem"
            @click="itemToDelete = currentItem; showDeleteItemConfirm = true">
            {{ t('collectionDetail.deleteItemButton') }}
          </button>
          <button class="btn-cancel" :disabled="isUpdatingItem" @click.stop="cancelEditItemModal()">
            {{ t('collectionDetail.cancel') }}
          </button>
          <button v-if="itemType === 'custom' || itemType === 'quiz'" :disabled="!customContent.front.trim() || !customContent.back.trim() || isUpdatingItem"
            class="btn-insert" @click="addCustomContent">
            {{ isEditingItem ? t('collectionDetail.updateItem') : t('collectionDetail.addItemButton') }}
          </button>
          <button v-else-if="itemType === 'definition' && selectedDefinition" class="btn-insert"
            :disabled="isUpdatingItem"
            @click="addNewItem(selectedDefinition)">
            {{ isEditingItem ? t('collectionDetail.updateItem') : t('collectionDetail.addSelectedDefinition') }}
          </button>
        <button v-else-if="itemType === 'newDefinition'" class="btn-insert"
          :disabled="isSubmittingNewDefinition || !newDefinitionData.word || !newDefinitionData.langId || (!newDefinitionData.definition && !newDefinitionImage) || !newDefinitionWordType"
          @click="addNewDefinitionAndItem">
          {{ isSubmittingNewDefinition ? t('collectionDetail.adding') : t('collectionDetail.addNewDefinitionButton') }}
        </button>
        </div>
      </div>
    </ModalComponent>

    <!-- Merge Collection Modal -->
    <ModalComponent :show="showMergeModal" :title="t('collectionDetail.mergeCollectionTitle')"
      @close="showMergeModal = false; showActions = !showActions">
      <div v-if="isLoadingCollections" class="flex justify-center py-4">
        <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500" />
      </div>

      <form v-else class="space-y-4" @submit.prevent="performMerge">
        <!-- Collection Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{
            t('collectionDetail.selectCollectionToMergeLabel')
          }}</label>
          <select v-model="selectedCollectionToMerge" class="input-field w-full" required>
            <option value="">
              {{ t('collectionDetail.selectCollectionPlaceholder') }}
            </option>
            <option v-for="c in availableCollections" :key="c.collection_id" :value="c.collection_id">
              {{ c.name }} ({{ t('collectionDetail.itemsCount', { count: c.item_count }) }})
            </option>
          </select>
        </div>

        <!-- New Collection Option -->
        <div class="flex items-center space-x-2">
          <input id="createNew" v-model="createNewCollection" type="checkbox" class="checkbox-toggle">
          <label for="createNew" class="text-sm text-gray-700">
            {{ t('collectionDetail.createNewFromMergeLabel') }}
          </label>
        </div>

        <!-- New Collection Name (if creating new) -->
        <div v-if="createNewCollection">
          <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionDetail.newCollectionNameLabel')
          }}</label>
          <input v-model="newMergedCollectionName" type="text" required class="input-field w-full"
            :placeholder="t('collectionDetail.newCollectionNamePlaceholder')">
        </div>

        <!-- Action Buttons -->
        <div class="flex justify-end space-x-3 pt-4">
          <button type="button" class="btn-cancel" @click="showMergeModal = false; showActions = !showActions">
            {{ t('collectionDetail.cancel') }}
          </button>
          <button type="submit" :disabled="!isValidMerge || isMerging" class="btn-update">
            {{ isMerging ? t('collectionDetail.merging') : t('collectionDetail.mergeButton') }}
          </button>
        </div>
      </form>
    </ModalComponent>
    <!-- Edit Notes Modal -->
  </div>

  <DeleteConfirmationModal :show="showDeleteItemConfirm" :title="t('collectionDetail.deleteItemConfirmTitle')"
    class="z-[65]"
    :message="t('collectionDetail.deleteItemConfirmMessage', { item: itemToDelete?.word || itemToDelete?.free_content_front })"
    :is-deleting="isDeletingItem" @confirm="removeItem(itemToDelete?.item_id)"
    @cancel="showDeleteItemConfirm = false" />

  <DeleteConfirmationModal :show="showDeleteCollectionConfirm" :title="t('collectionDetail.confirmDeleteTitle')"
    class="z-[65]" :message="t('collectionDetail.confirmDeleteMessage')" :is-deleting="isDeletingCollection"
    @confirm="performDeleteCollection" @cancel="showDeleteCollectionConfirm = false" />
</template>
<script setup>
import {
  GalleryHorizontalIcon,
  LayoutPanelTop,
  EllipsisVertical,
  BookOpen,
  Edit3,
  HelpCircle,
  PlusCircle,
  FilePlus,
  Search as SearchIcon,
  Loader,
  ArrowRight,
} from 'lucide-vue-next'
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import {
  getCollection,
  getCollections,
  updateCollection,
  deleteCollection,
  addCollectionItem,
  removeCollectionItem,
  getDefinition,
  searchDefinitions,
  listCollectionItems,
  mergeCollection,
  cloneCollection,
  importCollectionFull,
  getCollections as getUserCollections,
  getLanguages,
  updateItemPosition,
  exportCollectionFull,
  searchItems,
  getItemImage,
  deleteFlashcard,
  addValsi,
  analyzeWord,
  validateMathJax,
} from '@/api'
import AlertComponent from '@/components/AlertComponent.vue'
import DefinitionCard from '@/components/DefinitionCard.vue'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import DynamicInput from '@/components/DynamicInput.vue'
import ImageUpload from '@/components/ImageUpload.vue'
import SoundUpload from '@/components/SoundUpload.vue'
import ToastFloat from '@/components/ToastFloat.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SearchInput from '@/components/SearchInput.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'
import { SearchQueue } from '@/utils/searchQueue'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t, locale } = useI18n();

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
})

const route = useRoute()
const router = useRouter()
const auth = useAuth()

// PaginationComponent state
const currentPage = ref(parseInt(route.query.page) || 1)
const itemsPerPage = 10

// Check for flashcard mode
const isAddFlashcardMode = computed(() => route.query.mode === 'add_flashcard')


const editForm = ref({
  name: '',
  description: '',
  is_public: false,
})


// State
const collection = ref(null)
const { showError, clearError } = useError()
const isLoading = ref(true)
const isLoadingItems = ref(false)
const showActions = ref(false)
const showEditModal = ref(false)
const showAddModal = ref(false)
const isEditingItem = ref(false)
const currentItem = ref(null)
const editingItem = ref(null)

const numericCollectionId = computed(() => Number(props.collectionId))

const showExportModal = ref(false)
const isExporting = ref(false)
const exportError = ref('')
const exportProgress = ref('')
const showValidation = ref(false)

const ITEM_TYPE_STORAGE_KEY = 'collectionDetail_itemType'

const addItemTabs = computed(() => [
  {
    key: 'definition',
    label: t('collectionDetail.dictionaryDefinitionsTab'),
    icon: BookOpen,
  },
  {
    key: 'newDefinition',
    label: t('collectionDetail.newDefinitionTab'),
    icon: FilePlus,
  },
  {
    key: 'custom',
    label: t('collectionDetail.customContentTab'),
    icon: Edit3,
  },
])

// Computed properties
const isOwner = computed(() => {
  return collection.value?.owner?.username === auth.state.username
})

if (isOwner.value) { // Only add Quiz tab if user is owner
  addItemTabs.value.push({
    key: 'quiz', label: t('collectionDetail.quizTab'), icon: HelpCircle 
  });
}

const itemType = ref((typeof window === 'undefined') ? '' : localStorage.getItem(ITEM_TYPE_STORAGE_KEY) || 'definition')
watch(itemType, (newType) => {
  if (typeof window === 'undefined') return;

  localStorage.setItem(ITEM_TYPE_STORAGE_KEY, newType)
}, { immediate: true })

const customContent = ref({
  front: '',
  back: '',
  frontImage: null,
  backImage: null,
  sound: null,
  removeSound: false,
  auto_progress: true,
})

const definitionBackImage = ref(null)
const definitionSound = ref(null)
const definitionRemoveSound = ref(false)
const addItemDirection = ref('direct')
const useCanonicalComparison = ref(true)
const enableFlashcard = ref(false)
const customContentContainer = ref(null)
const searchContentContainer = ref(null)
const newDefinitionContainer = ref(null)

// --- New Definition State ---
const newDefinitionData = ref({
  word: '',
  langId: '',
  definition: '',
  notes: '',
  etymology: '',
  ownerOnly: false,
})
const newDefinitionImage = ref(null)
const newDefinitionSound = ref(null)
const newDefinitionWordType = ref('')
const newDefinitionRecommended = ref('')
const newDefinitionProblems = ref({})
const isAnalyzingNewWord = ref(false)
const newDefinitionError = ref('')
const newDefinitionSuccess = ref(false)
const isSubmittingNewDefinition = ref(false)
// --- End New Definition State ---


watch(enableFlashcard, (newVal) => {
  if (newVal) {
    nextTick(() => {
      if (customContentContainer.value) {
        customContentContainer.value.scrollTop = customContentContainer.value.scrollHeight
      } else if (quizContentContainer.value) {
        quizContentContainer.value.scrollTop = quizContentContainer.value.scrollHeight
      } else if (searchContentContainer.value) {
        searchContentContainer.value.scrollTop = searchContentContainer.value.scrollHeight
      } else if (newDefinitionContainer.value) {
        newDefinitionContainer.value.scrollTop = newDefinitionContainer.value.scrollHeight
      }
    })
  }
})

const handleDefinitionBackImageLoaded = (imageData) => {
  definitionBackImage.value = imageData
}

// --- New Definition Logic ---
const handleNewDefinitionImageLoaded = (imageObj) => {
  newDefinitionImage.value = imageObj
}

const handleNewDefinitionRemoveImage = () => {
  newDefinitionImage.value = null
}

const clearNewDefinitionAnalysis = () => {
  newDefinitionWordType.value = ''
  newDefinitionRecommended.value = ''
  newDefinitionProblems.value = {}
  clearError()
  newDefinitionSuccess.value = false
}

const useNewDefinitionRecommended = () => {
  if (newDefinitionRecommended.value) {
    newDefinitionData.value.word = newDefinitionRecommended.value
    newDefinitionRecommended.value = ''
  }
}

const doAnalyzeNewWord = async () => {
  if (!newDefinitionData.value.word) return
  newDefinitionData.value.word = newDefinitionData.value.word.trim()
  isAnalyzingNewWord.value = true

  try {
    const response = await analyzeWord(newDefinitionData.value.word)
    if (response.data.success) {
      newDefinitionSuccess.value = true
      clearError()
      newDefinitionWordType.value = response.data.word_type
      newDefinitionData.value.word = response.data.text // Update with potentially cleaned word
      newDefinitionRecommended.value = response.data.recommended && response.data.recommended !== newDefinitionData.value.word ? response.data.recommended : ''
      newDefinitionProblems.value = response.data.problems || {};
    } else {
      newDefinitionSuccess.value = false
      newDefinitionWordType.value = ''
      showError(t('upsertDefinition.analyzeError'))
    }
  } catch (e) {
    showError(t('upsertDefinition.analyzeErrorGeneric'))
  } finally {
    isAnalyzingNewWord.value = false
  }
}

const performValidateNewDefinitionMathJax = async () => {
  if (!newDefinitionData.value.definition) {
    newDefinitionError.value = ''
    return
  }
  try {
    await validateMathJax(newDefinitionData.value.definition)
    newDefinitionError.value = ''
  } catch (err) {
    newDefinitionError.value = err.response?.data?.error || t('upsertDefinition.validateError')
  }
}

const addNewDefinitionAndItem = async () => {
  if (!newDefinitionData.value.word || !newDefinitionData.value.langId || (!newDefinitionData.value.definition && !newDefinitionImage.value)) {
    showError(t('collectionDetail.newDefinitionValidationError'))
    return
  }
  if (!newDefinitionWordType.value) {
    showError(t('collectionDetail.analyzeWordFirstError'))
    return
  }

  await performValidateNewDefinitionMathJax()
  if (newDefinitionError.value) return

  isSubmittingNewDefinition.value = true
  clearError()

  try {
    // 1. Add the new definition (valsi)
    const addValsiPayload = {
      word: newDefinitionData.value.word,
      definition: newDefinitionData.value.definition,
      notes: newDefinitionData.value.notes || null,
      etymology: newDefinitionData.value.etymology || null,
      lang_id: parseInt(newDefinitionData.value.langId),
      owner_only: newDefinitionData.value.ownerOnly,
      image: newDefinitionImage.value,
      // Keywords are omitted as requested
    }
    const addValsiResponse = await addValsi(addValsiPayload)

    if (!addValsiResponse.data.success) {
      throw new Error(addValsiResponse.data.error || t('collectionDetail.addDefinitionFailed'))
    }

    const newDefinitionId = addValsiResponse.data.definition_id

    // 2. Add the new definition to the collection
    const addItemPayload = {
      definition_id: newDefinitionId,
      notes: addItemNotes.value, // Use the general notes field for the collection item
      auto_progress: customContent.value.auto_progress, // Use the general auto_progress
      direction: enableFlashcard.value ? addItemDirection.value : null, // Use general flashcard settings
      sound: newDefinitionSound.value,
      // Images are handled by the definition itself, not duplicated here
    }
    await addCollectionItem(props.collectionId, addItemPayload)

    // 3. Reset and refresh
    resetNewDefinitionForm()
    showAddModal.value = false
    await Promise.all([fetchItems(), fetchCollection()])

  } catch (error) {
    console.error('Error adding new definition and item:', error)
    showError(error.message || t('collectionDetail.addDefinitionItemFailed'))
  } finally {
    isSubmittingNewDefinition.value = false
  }
}

const resetNewDefinitionForm = () => {
  newDefinitionData.value = {
    word: '',
    langId: '',
    definition: '',
    notes: '',
    etymology: '',
    ownerOnly: false,
  }
  newDefinitionImage.value = null
  newDefinitionSound.value = null
  newDefinitionWordType.value = ''
  newDefinitionRecommended.value = ''
  newDefinitionProblems.value = {}
  newDefinitionError.value = ''
  newDefinitionSuccess.value = false
  isAnalyzingNewWord.value = false
  isSubmittingNewDefinition.value = false
}

// Watch for new definition changes to validate MathJax
let newDefinitionValidationTimeout = null
watch(() => newDefinitionData.value.definition, () => {
  if (newDefinitionValidationTimeout) {
    clearTimeout(newDefinitionValidationTimeout)
  }
  newDefinitionValidationTimeout = setTimeout(() => {
    performValidateNewDefinitionMathJax()
  }, 500)
})
// --- End New Definition Logic ---


const toggleFlashcard = async () => {
  if (enableFlashcard.value) {
    // If disabling, try to delete existing flashcard
    if (isEditingItem.value && currentItem.value?.flashcard?.id) {
      try {
        await deleteFlashcard(currentItem.value.flashcard.id)
      } catch (error) {
        if (error.response?.status !== 404) { // Ignore 404 errors
          console.error('Error deleting flashcard:', error)
        }
      }
    }
  }
  enableFlashcard.value = !enableFlashcard.value
}

const handleFrontImageLoaded = (imageData) => {
  customContent.value.frontImage = imageData
}

const handleBackImageLoaded = (imageData) => {
  customContent.value.backImage = imageData
}

const addCustomContent = async () => {
  showValidation.value = true
  if (!customContent.value.front.trim() || !customContent.value.back.trim()) {
    return
  }

  if (isEditingItem.value) {
    isUpdatingItem.value = true
  }

  try {
    const direction = itemType.value === 'quiz' ? addItemDirection.value : (enableFlashcard.value ? addItemDirection.value : null);

    await addCollectionItem(props.collectionId, {
      item_id: isEditingItem.value ? currentItem.value?.item_id : undefined,
      free_content_front: customContent.value.front.trim(),
      free_content_back: customContent.value.back.trim(),
      notes: addItemNotes.value,
      front_image: customContent.value.frontImage,
      back_image: customContent.value.backImage,
      sound: customContent.value.sound,
      remove_sound: customContent.value.removeSound,
      auto_progress: itemType.value === 'quiz' ? true : customContent.value.auto_progress, // Quizzes are always auto-progress for now
      direction: direction,
      use_canonical_comparison: useCanonicalComparison.value,
    })

    showAddModal.value = false
    const hadSoundChange = !!customContent.value.sound || !!customContent.value.removeSound
    if (hadSoundChange) {
      successMessage.value = t('collectionDetail.itemUpdatedSoundSaved', 'Item updated. Sound saved.')
      showSuccessToast.value = true
      setTimeout(() => { showSuccessToast.value = false }, 3000)
    }

    customContent.value = {
      front: '',
      back: '',
      frontImage: null,
      backImage: null,
      sound: null,
      removeSound: false,
    }

    addItemNotes.value = ''
    showValidation.value = false

    await Promise.all([fetchItems(), fetchCollection()])

    if (editItemId.value) {
      router.push(`/collections/${props.collectionId}/flashcards`)
    }
  } catch (error) {
    console.error('Error adding custom content:', error)
  } finally {
    isUpdatingItem.value = false
  }
}

const handleExport = async () => {
  if (isExporting.value) return

  isExporting.value = true
  exportError.value = ''
  exportProgress.value = 'Preparing export...'

  try {
    exportProgress.value = 'Requesting export...'
    const response = await exportCollectionFull(props.collectionId)
    exportProgress.value = 'Processing response...'

    const data = response.data
    const filename = `${(collection.value?.name || 'collection').replace(/[^a-zA-Z0-9-_]/g, '_')}-export.json`
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    window.URL.revokeObjectURL(url)
    a.remove()

    exportProgress.value = 'Export complete!'
    setTimeout(() => {
      showExportModal.value = false
    }, 1000)
  } catch (err) {
    exportProgress.value = ''
    exportError.value = err.response?.data?.error || err.message || 'Export failed'
  } finally {
    isExporting.value = false
    showExportModal.value = false
  }
}

const openEditItemModal = async (item) => {
  editingItem.value = item
  isEditingItem.value = true
  showAddModal.value = true

  await nextTick(() => {
    if (customContentContainer.value) {
      customContentContainer.value.scrollTop = customContentContainer.value.scrollHeight
    }
  })

  // Initialize common fields
  addItemNotes.value = item.ci_notes
  enableFlashcard.value = !!item.flashcard
  addItemDirection.value = item.flashcard?.direction || 'direct'
  customContent.value.auto_progress = item.auto_progress
  currentItem.value = item
  definitionSound.value = null
  definitionRemoveSound.value = false

  // Load existing content based on item type
  if (item.definition_id) {
    // Definition-based item - populate definition tab
    itemType.value = 'definition'
    try {
      const response = await getDefinition(item.definition_id)
      if (response.data) {
        // Set selected definition with full data
        const def = response.data
        selectedDefinition.value = {
          ...def, // Spread definition properties
          free_content_front: item.free_content_front,
          free_content_back: item.free_content_back,
          has_front_image: item.has_front_image,
          has_back_image: item.has_back_image
        }

        searchQuery.value = normalizeSearchQuery(def.valsiword) // Keep setting search query for context

        addItemResults.value = [def] // Keep setting results for consistency, though only one is selected

        // Initialize images
        const initImage = async (side) => {
          if (item[`has_${side}_image`]) {
            try {
              const response = await getItemImage(props.collectionId, item.item_id, side)
              const blob = new Blob([response.data], { type: response.headers['content-type'] })

              // Convert blob to base64
              return new Promise((resolve) => {
                const reader = new FileReader()
                reader.onload = () => {
                  const base64 = reader.result.split(',')[1] // Get only the base64 part
                  resolve({
                    data: base64,
                    mime_type: response.headers['content-type']
                  })
                }
                reader.readAsDataURL(blob)
              })
            } catch (error) {
              console.error(`Error loading ${side} image:`, error)
            }
          }
          return null
        }

        definitionBackImage.value = await initImage('back')
      }
    } catch (error) {
      console.error('Error loading definition for edit:', error)
    }
  } else if (item.free_content_front || item.free_content_back) {
    // Custom content or Quiz item
    if (item.flashcard?.direction?.toLowerCase().includes('quiz')) {
      itemType.value = 'quiz';
      addItemDirection.value = item.flashcard.direction; // Pre-select quiz direction
    } else {
      itemType.value = 'custom';
      // For custom non-quiz, set enableFlashcard and addItemDirection based on existing flashcard
      enableFlashcard.value = !!item.flashcard;
      addItemDirection.value = item.flashcard?.direction || 'direct';
    }

    customContent.value = {
      front: item.free_content_front || '',
      back: item.free_content_back || '',
      frontImage: null,
      backImage: null,
      sound: null,
      removeSound: false,
      auto_progress: itemType.value === 'quiz' ? true : item.auto_progress,
    }

    // Load existing images for custom content
    const loadImage = async (side) => {
      if (item[`has_${side}_image`]) {
        try {
          const response = await getItemImage(props.collectionId, item.item_id, side)
          const blob = new Blob([response.data], { type: response.headers['content-type'] })
          // Convert to base64 data URI
          return new Promise((resolve) => {
            const reader = new FileReader()
            reader.onload = () => {
              resolve({
                data: reader.result.split(',')[1],
                mime_type: response.headers['content-type'],
                dataUri: reader.result
              })
            }
            reader.readAsDataURL(blob)
          })
        } catch (error) {
          console.error(`Error loading ${side} image:`, error)
        }
      }
      return null
    }

    // Create full data URI for preview
    const frontImage = await loadImage('front')
    if (frontImage) {
      customContent.value.frontImage = {
        data: frontImage.data,
        mime_type: frontImage.mime_type,
        dataUri: frontImage.dataUri
      }
    }
    // Create full data URI for preview
    const backImage = await loadImage('back')
    if (backImage) {
      customContent.value.backImage = {
        data: backImage.data,
        mime_type: backImage.mime_type,
        dataUri: backImage.dataUri
      }
    }

    // Load existing images if available
    if (item.has_front_image) {
      try {
        const response = await getItemImage(props.collectionId, item.item_id, 'front')
        const blob = new Blob([response.data], { type: response.headers['content-type'] })
        customContent.value.frontImage = {
          data: URL.createObjectURL(blob),
          mime_type: response.headers['content-type']
        }
      } catch (error) {
        console.error('Error loading front image:', error)
      }
    }

    if (item.has_back_image) {
      try {
        const response = await getItemImage(props.collectionId, item.item_id, 'back')
        const blob = new Blob([response.data], { type: response.headers['content-type'] })

        customContent.value.backImage = await new Promise((resolve) => {
          const reader = new FileReader()
          reader.onload = () => {
            resolve({
              data: reader.result.split(',')[1],
              mime_type: response.headers['content-type'],
              dataUri: reader.result
            })
          }
          reader.readAsDataURL(blob)
        })
      } catch (error) {
        console.error('Error loading back image:', error)
      }
    }
  }
}

function cancelEditCollectionModal() {
  showEditModal.value = false
  // Reset edit form if needed, or handle potential unsaved changes
  // For now, just closing the modal.
}

function cancelEditItemModal() {
  if (editItemId.value) {
    router.push(`/collections/${props.collectionId}/flashcards`)
  } else {
    showAddModal.value = false
    isEditingItem.value = false
    currentItem.value = null
  }

}

const showDeleteItemConfirm = ref(false)
const itemToDelete = ref(null)
const isDeletingItem = ref(false)
const isUpdatingItem = ref(false)
const showDeleteCollectionConfirm = ref(false) // New state for collection deletion confirmation
const isDeletingCollection = ref(false) // New state for collection deletion loading
const showSuccessToast = ref(false)
const isImportingJson = ref(false) // State for JSON import loading

const isSubmitting = ref(false)
// Search state
const searchQuery = ref('')
const itemSearchQuery = ref('')
const addItemResults = ref([])
const isSearching = ref(false)
let searchTimeoutFilter = null

// Debounce delay: 450ms is optimal for search inputs (400-500ms range)
// This balances responsiveness with reducing unnecessary API calls
const DEBOUNCE_DELAY = 450

// Search queues to prevent race conditions
const itemsSearchQueue = new SearchQueue()
const definitionsSearchQueue = new SearchQueue()

function clearSearchTimeoutFilter() {
  if (searchTimeoutFilter) {
    clearTimeout(searchTimeoutFilter)
    searchTimeoutFilter = null
  }
  isSearching.value = false
}

const handleSearch = () => {
  // Clear any pending timeouts to prevent stale searches
  clearSearchTimeoutFilter()
  
  // Capture current query value to check in timeout
  const currentQuery = itemSearchQuery.value
  
  // Debounce the search - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  searchTimeoutFilter = setTimeout(() => {
    // Only perform search if query hasn't changed (to prevent race conditions)
    if (itemSearchQuery.value === currentQuery) {
      // Show loading spinner when search actually starts
      if (itemSearchQuery.value.trim()) {
        isSearching.value = true
      }
      performSearch()
    }
    searchTimeoutFilter = null
  }, DEBOUNCE_DELAY)
}

const performSearch = async () => {
  let requestId = null
  
  try {
    if (!itemSearchQuery.value.trim()) {
      // If search is empty, fetch regular items
      await fetchItems()
      return
    }

    const request = itemsSearchQueue.createRequest()
    requestId = request.requestId
    const { signal } = request

    const response = await searchItems({
      q: normalizeSearchQuery(itemSearchQuery.value),
      user_id: collection.value?.owner?.user_id,
    }, signal)

    // Only process if this is still the latest request
    if (!itemsSearchQueue.shouldProcess(requestId)) {
      return
    }

    paginatedItems.value = {
      items: response.data.items,
      total: response.data.total,
      page: 1,
      per_page: itemsPerPage,
    }
  } catch (error) {
    // Ignore abort errors
    if (error.name === 'AbortError' || error.code === 'ERR_CANCELED' || error.message?.includes('canceled')) {
      return
    }
    
    console.error('Error searching items:', error)
  } finally {
    // Only update loading state if this is still the latest request
    if (requestId && itemsSearchQueue.shouldProcess(requestId)) {
      isSearching.value = false
    } else if (!itemsSearchQueue.hasActiveRequest()) {
      isSearching.value = false
    }
  }
}

const addItemNotes = ref('')
const paginatedItems = ref({ items: [], total: 0 })
const isCloning = ref(false)
const isReordering = ref(false)
const languages = ref([])
const userCollections = ref([])

const editItemId = computed(() => route.query.editItem)

const validateAccess = (collection) => {
  if (
    !collection.is_public &&
    (!auth.state.isLoggedIn || collection.owner.username !== auth.state.username)
  ) {
    router.push('/collections')
    return false
  }
  return true
}

const totalPages = computed(() => {
  return Math.ceil(paginatedItems.value.total / itemsPerPage)
})

const handleCloneCollection = async () => {
  isCloning.value = true
  try {
    const response = await cloneCollection(props.collectionId)
    router.push(`/collections/${response.data.collection_id}`)
  } catch (error) {
    console.error('Error cloning collection:', error)
  } finally {
    isCloning.value = false
    showActions.value = false
  }
}

// Fetch user's collections
const fetchUserCollections = async () => {
  if (!auth.state.isLoggedIn) return;
  try {
    const response = await getUserCollections();
    userCollections.value = response.data.collections;
  } catch (error) {
    console.error("Error fetching user collections:", error);
  }
};

// Fetch collection details
const fetchCollection = async () => {
  isLoading.value = true
  clearError();

  try {
    const response = await getCollection(props.collectionId)
    collection.value = response.data

    if (!validateAccess(collection.value)) return

    // Initialize edit form
    editForm.value = {
      name: collection.value.name,
      description: collection.value.description || '',
      is_public: collection.value.is_public,
    }

    // Fetch first page of items
    await fetchItems()
    const langsResponse = await getLanguages()
    languages.value = langsResponse.data
  } catch (e) {
    showError(e.response?.data?.error || 'Failed to load collection')
    console.error('Error fetching collection:', e)
  } finally {
    isLoading.value = false
  }
}

// Fetch paginated items
const fetchItems = async () => {
  isLoadingItems.value = true

  try {
    const response = await listCollectionItems(props.collectionId, {
      page: currentPage.value,
      per_page: itemsPerPage,
      search: normalizeSearchQuery(itemSearchQuery.value).trim() || undefined,
      item_id: editItemId.value,
      exclude_with_flashcards: isAddFlashcardMode.value || undefined,
    })
    paginatedItems.value = response.data

    // Check if we need to open edit modal from query param
    if (editItemId.value) {
      const item = paginatedItems.value.items.find(i => i.item_id == editItemId.value)
      if (item) {
        openEditItemModal(item)
      }
    }
  } catch (e) {
    console.error('Error fetching items:', e)
  } finally {
    isLoadingItems.value = false
  }
}

const moveItem = async (item, direction) => {
  if (isReordering.value) return
  isReordering.value = true

  const items = [...paginatedItems.value.items]
  const currentIndex = items.findIndex((i) => i.item_id === item.item_id)
  const newIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1
  const targetItem = items[newIndex]

  try {
    // Calculate new position
    const newPosition = targetItem.position

    // Optimistically update UI
    const [movedItem] = items.splice(currentIndex, 1)
    items.splice(newIndex, 0, movedItem)

    // Update positions
    items.forEach((item, idx) => {
      item.position = idx + 1
    })

    paginatedItems.value = {
      ...paginatedItems.value,
      items,
    }

    // Update server
    await updateItemPosition(props.collectionId, item.item_id, newPosition)
  } catch (error) {
    // Revert on error
    await fetchItems()
    console.error('Failed to reorder item:', error)
  } finally {
    isReordering.value = false
  }
}

// Navigation
const prevPage = () => {
  if (currentPage.value > 1) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value - 1,
      },
    })
  }
}

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value + 1,
      },
    })
  }
}

// Sync page from URL
const syncFromRoute = () => {
  currentPage.value = parseInt(route.query.page) || 1
}

watch(
  () => route.query.page,
  (newPage) => {
    const pageNum = parseInt(newPage) || 1
    if (pageNum !== currentPage.value) {
      syncFromRoute()
      fetchItems()
    }
  }
)

watch(
  () => route.query.mode,
  () => {
    fetchItems()
  }
)

// Update collection
const performUpdateCollection = async () => {
  if (isSubmitting.value) return
  isSubmitting.value = true

  try {
    const response = await updateCollection(props.collectionId, editForm.value)
    collection.value = response.data
    showEditModal.value = false
  } catch (error) {
    console.error('Error updating collection:', error)
  } finally {
    isSubmitting.value = false
  }
}

// Show confirmation modal for deleting collection
const handleDelete = () => {
  showDeleteCollectionConfirm.value = true
}

// Perform collection deletion after confirmation
const performDeleteCollection = async () => {
  isDeletingCollection.value = true
  try {
    await deleteCollection(props.collectionId)
    router.push('/collections')
  } catch (error) {
    console.error('Error deleting collection:', error)
    showError(error.response?.data?.error || 'Failed to delete collection')
  } finally {
    isDeletingCollection.value = false
    showDeleteCollectionConfirm.value = false
  }
}

// Search debouncing
let searchTimeout = null

function clearSearchTimeout() {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
    searchTimeout = null
  }
  isSearching.value = false
}

const debouncedSearch = () => {
  // Clear any pending timeouts to prevent stale searches
  clearSearchTimeout()
  
  // Capture current query value to check in timeout
  const currentQuery = searchQuery.value
  
  // Debounce the search - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  searchTimeout = setTimeout(() => {
    // Only perform search if query hasn't changed (to prevent race conditions)
    if (searchQuery.value === currentQuery) {
      // Show loading spinner when search actually starts
      if (searchQuery.value.trim()) {
        isSearching.value = true
      }
      performSearchDefinitions()
    }
    searchTimeout = null
  }, DEBOUNCE_DELAY)
}

// Search definitions for adding
const performSearchDefinitions = async () => {
  if (!searchQuery.value) {
    addItemResults.value = []
    isSearching.value = false
    return
  }

  let requestId = null
  const request = definitionsSearchQueue.createRequest()
  requestId = request.requestId
  const { signal } = request

  try {
    const response = await searchDefinitions({ search: searchQuery.value }, signal)
    
    // Only process if this is still the latest request
    if (!definitionsSearchQueue.shouldProcess(requestId)) {
      return
    }
    
    addItemResults.value = response.data.definitions
  } catch (error) {
    // Ignore abort errors
    if (error.name === 'AbortError' || error.code === 'ERR_CANCELED' || error.message?.includes('canceled')) {
      return
    }
    
    // Only show errors for the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      console.error('Error searching definitions:', error)
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (requestId && definitionsSearchQueue.shouldProcess(requestId)) {
      isSearching.value = false
    } else if (!definitionsSearchQueue.hasActiveRequest()) {
      isSearching.value = false
    }
  }
}

const clearItemSearch = () => {
  // Clear any pending timeouts first to prevent them from firing after clearing
  clearSearchTimeoutFilter()
  itemSearchQuery.value = ''
  fetchItems()
}

const clearDefinitionSearch = () => {
  // Clear any pending timeouts first to prevent them from firing after clearing
  clearSearchTimeout()
  selectedDefinition.value = ''
  searchQuery.value = ''
  addItemResults.value = []
}

// Add definition to collection
const selectedDefinition = ref(null)

const selectDefinition = async (definition) => {
  if (selectedDefinition.value?.definitionid === definition.definitionid) {
    selectedDefinition.value = null
  } else {
    selectedDefinition.value = definition
    await nextTick()
    const selectedEl = document.querySelector(`[data-definition-id="${definition.definitionid}"]`)
    selectedEl?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }
}

const resetForm = () => {
  if (editItemId.value) {
    router.push(`/collections/${props.collectionId}/flashcards`)
  } else {
    showAddModal.value = false
  }

  addItemNotes.value = ''
  searchQuery.value = ''
  addItemResults.value = []
  definitionBackImage.value = null
  definitionSound.value = null
  definitionRemoveSound.value = false
  selectedDefinition.value = null
  enableFlashcard.value = false
  addItemDirection.value = 'direct'
  isEditingItem.value = false
  currentItem.value = null
  customContent.value = {
    front: '',
    back: '',
    frontImage: null,
    backImage: null,
    sound: null,
    removeSound: false,
    auto_progress: true,
  }
}

const addNewItem = async (definition) => {
  if (isEditingItem.value) {
    isUpdatingItem.value = true
  }
  try {
    const payload = {
      collection_id: numericCollectionId.value,
      definition_id: definition.definitionid,
      free_content_front: definition.free_content_front,
      free_content_back: definition.free_content_back,
      notes: addItemNotes.value,
      front_image: itemType.value === 'definition' ? null : customContent.value.frontImage, // Only for custom/quiz
      back_image: definitionBackImage.value,
      sound: definitionSound.value,
      remove_sound: definitionRemoveSound.value,
      auto_progress: customContent.value.auto_progress,
      direction: enableFlashcard.value ? addItemDirection.value : null
    }


    await addCollectionItem(props.collectionId, payload)

    resetForm()
    await Promise.all([fetchItems(), fetchCollection()])
  } catch (error) {
    console.error('Error adding definition:', error)
  } finally {
    isUpdatingItem.value = false
  }
}

const confirmRemoveItem = (item) => {
  itemToDelete.value = item
  showDeleteItemConfirm.value = true
}

const removeItem = async (itemId) => {
  if (!itemId) return

  isDeletingItem.value = true
  try {
    await removeCollectionItem(props.collectionId, itemId)
    await Promise.all([fetchItems(), fetchCollection()])

    // If deleted item was last one on page and not first page, go to previous page
    if (paginatedItems.value.items.length === 0 && currentPage.value > 1) {
      currentPage.value -= 1
      router.push({
        query: {
          ...route.query,
          page: currentPage.value
        }
      })
      await fetchItems()
    }

    clearError()
    showAddModal.value = false
  } catch (err) {
    showError(err.response?.data?.error || 'Failed to remove item')
  } finally {
    isDeletingItem.value = false
    showDeleteItemConfirm.value = false
    itemToDelete.value = null
  }
}

// Document click handler for closing dropdowns
const toggleAddFlashcardMode = () => {
  const newQuery = {
    ...route.query,
    mode: undefined
  }

  router.push({
    query: newQuery
  })
}

const handleDocumentClick = (event) => {
  if (showActions.value && !event.target.closest('.actions-dropdown')) {
    showActions.value = false
  }
}

const successMessage = ref('')
const showMergeModal = ref(false)
const selectedCollectionToMerge = ref('')
const createNewCollection = ref(false)
const newMergedCollectionName = ref('')
const isLoadingCollections = ref(false)
const isMerging = ref(false)
const availableCollections = ref([])

const isValidMerge = computed(() => {
  if (!selectedCollectionToMerge.value) return false
  if (createNewCollection.value && !newMergedCollectionName.value.trim()) return false
  return selectedCollectionToMerge.value !== props.collectionId // Can't merge with self
})

const loadAvailableCollections = async () => {
  isLoadingCollections.value = true
  try {
    const response = await getCollections()
    // Filter out the current collection
    availableCollections.value = response.data.collections.filter(
      (c) => c.collection_id !== parseInt(props.collectionId)
    )
  } catch (e) {
    showError('Failed to load collections')
    console.error('Error loading collections:', e)
  } finally {
    isLoadingCollections.value = false
  }
}

// Add the merge function
const performMerge = async () => {
  if (!isValidMerge.value || isMerging.value) return

  isMerging.value = true
  clearError();

  try {
    await mergeCollection({
      source_collection_id: parseInt(selectedCollectionToMerge.value),
      target_collection_id: parseInt(props.collectionId),
      new_collection_name: createNewCollection.value ? newMergedCollectionName.value : undefined,
    })

    // If creating new collection, redirect to it
    if (createNewCollection.value) {
      router.push('/collections')
    } else {
      // Otherwise refresh the current collection
      await fetchCollection()
    }

    // Reset merge form
    showMergeModal.value = false
    selectedCollectionToMerge.value = ''
    createNewCollection.value = false
    newMergedCollectionName.value = ''

    // Show success message
    successMessage.value = t('collectionDetail.mergeSuccess')
    showSuccessToast.value = true
    setTimeout(() => {
      successMessage.value = ''
    }, 3000)
  } catch (err) {
    console.error(err)
    showError(err.response?.data?.error || 'Failed to merge collections')
  } finally {
    isMerging.value = false
  }
}

// --- JSON Import Logic ---
const jsonImportInput = ref(null)

const triggerJsonImport = () => {
  jsonImportInput.value?.click()
  showActions.value = false // Close actions dropdown
}

const handleJsonFileSelect = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  isImportingJson.value = true
  successMessage.value = ''
  clearError()

  try {
    const fileContent = await file.text()
    const jsonData = JSON.parse(fileContent)

    // Full format: { collection, items, levels? }
    if (!jsonData.collection || !Array.isArray(jsonData.items)) {
      showError(t('collectionDetail.importFullFormatError', 'Use a full collection export file (JSON with collection, items, and optionally levels).'))
      return
    }

    const payload = {
      collection: {
        name: jsonData.collection.name,
        description: jsonData.collection.description ?? null,
        is_public: jsonData.collection.is_public ?? true,
      },
      items: jsonData.items,
      levels: Array.isArray(jsonData.levels) ? jsonData.levels : [],
    }

    const response = await importCollectionFull(payload)
    const { collection: newCollection, imported_count, skipped_count, levels_created, warnings } = response.data

    let message = t('collectionDetail.importFullSuccess', {
      name: newCollection.name,
      imported: imported_count,
      skipped: skipped_count,
      levels: levels_created,
    }) || `Created "${newCollection.name}" with ${imported_count} items, ${levels_created} levels.`
    if (warnings?.length) {
      message += '<br/><br/>' + warnings.slice(0, 5).join('<br/>')
      if (warnings.length > 5) message += `<br/>... ${warnings.length - 5} more`
    }
    successMessage.value = message
    showSuccessToast.value = true
    setTimeout(() => {
      showSuccessToast.value = false
    }, 4000)
    router.push(`/collections/${newCollection.collection_id}`)
  } catch (error) {
    showError(error.response?.data?.error || t('collectionDetail.importJsonError', 'Import failed'))
  } finally {
    isImportingJson.value = false
    jsonImportInput.value.value = ''
  }
}
// --- End JSON Import Logic ---

onMounted(() => {
  fetchCollection();
  if (auth.state.isLoggedIn) {
    fetchUserCollections();
  }
  document.addEventListener('click', handleDocumentClick)
})

watch(() => auth.state.isLoggedIn, (loggedIn) => {
  if (loggedIn) {
    fetchUserCollections();
  } else {
    userCollections.value = [];
  }
});

useSeoHead({ title: computed(() => collection.value?.name || 'Collection') }, locale.value)

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
  // Clean up any pending search timeouts
  clearSearchTimeoutFilter()
  clearSearchTimeout()
})

watch(
  () => props.collectionId,
  () => {
    fetchCollection()
  }
)
</script>

<style scoped>
.prose :deep(p) {
  margin: 0;
}

@keyframes indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(400%);
  }
}

.progress-indeterminate {
  animation: indeterminate 1.2s ease-in-out infinite;
}
</style>
