<template>
  <div class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-4">{{ t('bulkImportClients.title') }}</h1>

    <div v-if="loadingClients" class="text-center">
      <p>{{ t('bulkImportClients.loadingGroups') }}</p>
      <!-- Add a spinner or loading animation if available -->
    </div>

    <div
      v-else-if="clientsError"
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4"
      role="alert"
    >
      <strong class="font-bold">{{ t('bulkImportClients.errorTitle') }}</strong>
      <span class="block sm:inline"> {{ clientsError }}</span>
    </div>

    <div v-else-if="clients.length === 0" class="text-center text-gray-500">
      {{ t('bulkImportClients.noGroups') }}
    </div>

    <div v-else>
      <ul class="space-y-4">
        <li v-for="client in clients" :key="client.client_id" class="border rounded p-4 shadow">
          <div
            class="flex justify-between items-center cursor-pointer"
            @click="toggleClient(client.client_id)"
          >
            <div class="space-x-1">
              <span class="italic">{{ t('bulkImportClients.clientIdLabel') }}</span>
              <span class="font-semibold">{{ client.client_id }}</span>
            </div>
            <span class="text-sm text-gray-600">{{
              t('bulkImportClients.definitionsCount', { count: client.count })
            }}</span>
            <button class="text-blue-500 hover:text-blue-700 text-sm">
              {{
                isExpanded(client.client_id)
                  ? t('bulkImportClients.collapse')
                  : t('bulkImportClients.expand')
              }}
            </button>
          </div>

          <div v-if="isExpanded(client.client_id)" class="mt-4 pt-4 border-t">
            <div v-if="expandedClients[client.client_id]?.loading" class="text-center text-sm">
              {{ t('bulkImportClients.loadingDefinitions') }}
            </div>
            <div
              v-else-if="expandedClients[client.client_id]?.error"
              class="bg-red-100 border border-red-400 text-red-700 px-3 py-2 rounded text-sm"
              role="alert"
            >
              {{ t('bulkImportClients.loadDefinitionsError') }}
              {{ expandedClients[client.client_id].error }}
            </div>
            <div
              v-else-if="expandedClients[client.client_id]?.definitions.length === 0"
              class="text-sm text-gray-500"
            >
              {{ t('bulkImportClients.noDefinitions') }}
            </div>
            <div v-else>
              <ul class="space-y-2">
                <li
                  v-for="def in expandedClients[client.client_id].definitions"
                  :key="def.id"
                  class="text-sm border-b pb-1"
                >
                  <span class="italic mr-2">{{ def.langrealname }}</span>
                  <span class="font-medium">{{ def.valsiword }}</span
                  >: {{ truncateDefinition(def.definition) }}
                  <!-- Add link to full definition page if needed -->
                  <!-- <router-link :to="{ name: 'valsi-detail', params: { id: def.valsi_id } }" class="text-blue-500 hover:underline ml-2">View</router-link> -->
                </li>
              </ul>
              <button
                v-if="expandedClients[client.client_id]?.hasMore"
                @click="loadMoreDefinitions(client.client_id)"
                :disabled="expandedClients[client.client_id]?.loadingMore"
                class="mt-3 px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 text-sm"
              >
                {{
                  expandedClients[client.client_id]?.loadingMore
                    ? t('bulkImportClients.loadingMore')
                    : t('bulkImportClients.loadMore')
                }}
              </button>
              <div
                v-if="expandedClients[client.client_id]?.loadMoreError"
                class="mt-2 bg-red-100 border border-red-400 text-red-700 px-3 py-2 rounded text-sm"
                role="alert"
              >
                {{ t('bulkImportClients.loadMoreError') }}
                {{ expandedClients[client.client_id].loadMoreError }}
              </div>
            </div>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSeoHead } from '@/composables/useSeoHead'
import { getBulkImportClients, getBulkImportClientDefinitions } from '@/api'

const { t } = useI18n()
useSeoHead({ title: t('bulkImportClients.title') })

const clients = ref([])
const loadingClients = ref(true)
const clientsError = ref(null)
const expandedClients = reactive({}) // Store expanded client data: { clientId: { definitions: [], loading: false, error: null, page: 1, perPage: 10, hasMore: true, loadingMore: false, loadMoreError: null } }

const ITEMS_PER_PAGE = 10 // Adjust as needed

const fetchClients = async () => {
  loadingClients.value = true
  clientsError.value = null
  try {
    const response = await getBulkImportClients()
    clients.value = response.data
  } catch (error) {
    console.error('Error fetching bulk import clients:', error)
    clientsError.value =
      error.response?.data?.detail || error.message || t('bulkImportClients.loadGroupsError')
  } finally {
    loadingClients.value = false
  }
}

const isExpanded = (clientId) => {
  return !!expandedClients[clientId]
}

const fetchClientDefinitions = async (clientId, page = 1) => {
  const clientData = expandedClients[clientId]
  if (!clientData) return // Should not happen if called correctly

  if (page === 1) {
    clientData.loading = true
    clientData.error = null
  } else {
    clientData.loadingMore = true
    clientData.loadMoreError = null
  }

  try {
    const response = await getBulkImportClientDefinitions(clientId, {
      page: page,
      per_page: ITEMS_PER_PAGE,
    })
    const newDefinitions = response.data.definitions || []

    if (page === 1) {
      clientData.definitions = newDefinitions
    } else {
      clientData.definitions.push(...newDefinitions)
    }

    clientData.page = page
    clientData.hasMore = newDefinitions.length === ITEMS_PER_PAGE
  } catch (error) {
    console.error(`Error fetching definitions for client ${clientId}, page ${page}:`, error)
    const errorMessage =
      error.response?.data?.detail || error.message || t('bulkImportClients.loadDefinitionsError')
    if (page === 1) {
      clientData.error = errorMessage
    } else {
      clientData.loadMoreError = errorMessage
    }
    // Keep existing definitions if loading more fails
  } finally {
    if (page === 1) {
      clientData.loading = false
    } else {
      clientData.loadingMore = false
    }
  }
}

const toggleClient = (clientId) => {
  if (isExpanded(clientId)) {
    // Collapse
    delete expandedClients[clientId]
  } else {
    // Expand: Initialize data structure and fetch first page
    expandedClients[clientId] = {
      definitions: [],
      loading: true,
      error: null,
      page: 1,
      perPage: ITEMS_PER_PAGE,
      hasMore: true,
      loadingMore: false,
      loadMoreError: null,
    }
    fetchClientDefinitions(clientId, 1)
  }
}

const loadMoreDefinitions = (clientId) => {
  const clientData = expandedClients[clientId]
  if (clientData && clientData.hasMore && !clientData.loadingMore) {
    fetchClientDefinitions(clientId, clientData.page + 1)
  }
}

const truncateDefinition = (text, length = 100) => {
  if (!text) return ''
  return text.length > length ? text.substring(0, length) + '...' : text
}

onMounted(() => {
  fetchClients()
})
</script>
