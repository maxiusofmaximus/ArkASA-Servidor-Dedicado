import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@ark-asa/shared-types': new URL('../packages/shared-types/src/index.ts', import.meta.url).pathname,
    },
  },
  server: {
    port: 5174, // desktop uses 5173
    host: true,
  },
})
