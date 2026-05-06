<script setup lang="ts">
/**
 * Primary “save / commit changes” affordance. Uses a document + check (not the legacy floppy-disk Save glyph).
 */
import { FileCheck } from 'lucide-vue-next'
</script>

<template><FileCheck aria-hidden="true" v-bind="$attrs" /></template>
