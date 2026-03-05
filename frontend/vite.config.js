import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@packages/ui': path.resolve(__dirname, './packages/ui'),
    },
  },
  build: {
    // Ensure consistent asset naming and proper chunking
    rollupOptions: {
      output: {
        // Ensure assets are properly hashed for cache busting
        assetFileNames: 'assets/[name]-[hash][extname]',
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
      },
    },
    // Increase chunk size warning limit if needed
    chunkSizeWarningLimit: 1000,
  },
})
