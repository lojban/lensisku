import path from 'node:path'
import { fileURLToPath } from 'node:url'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// https://vite.dev/config/
// Tailwind brand layers (`tailwind.*.mjs`): `pnpm dev` uses
// `scripts/dev-with-tailwind-reload.mjs` so edits restart Vite in a new process
// (Node ESM cache would otherwise keep stale `addComponents`).
export default defineConfig({
  plugins: [
    vue({
      template: {
        compilerOptions: {},
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@packages/ui': path.resolve(__dirname, './packages/ui'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        assetFileNames: 'assets/[name]-[hash][extname]',
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
      },
    },
    chunkSizeWarningLimit: 1000,
  },
})
