<template>

  <div class="bg-white p-4 rounded-lg border border-gray-200 animate-pulse">

    <div class="flex items-start space-x-3">
       <!-- Skeleton for avatar -->
      <div class="w-10 h-10 rounded-full bg-gray-200 flex-shrink-0"></div>

      <div class="flex-1 space-y-3">
         <!-- Skeleton for username and date -->
        <div class="flex justify-between items-center">

          <div class="h-4 bg-gray-200 rounded w-1/4"></div>

          <div class="h-3 bg-gray-200 rounded w-1/6"></div>

        </div>
         <!-- Skeleton for content -->
        <div class="space-y-2">

          <div class="h-4 bg-gray-200 rounded w-full"></div>

          <div class="h-4 bg-gray-200 rounded w-5/6"></div>

          <div class="h-4 bg-gray-200 rounded w-3/4"></div>

        </div>
         <!-- Skeleton for actions -->
        <div class="flex justify-end space-x-2 pt-2">

          <div class="h-6 bg-gray-200 rounded-full w-16"></div>

          <div class="h-6 bg-gray-200 rounded-full w-16"></div>

        </div>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
// No script logic needed for a simple skeleton
</script>

