import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    warmup: {
      clientFiles: ['./src/App.tsx', './src/main.tsx'],
    },
  },
  build: {
    target: 'esnext',
    cssMinify: 'lightningcss',
    minify: 'esbuild',
    sourcemap: false,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'zustand'
    ],
    exclude: ['@tauri-apps/api', '@tauri-apps/api/tauri', '@tauri-apps/cli']
  },
})
