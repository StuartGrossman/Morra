import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      'stream': 'stream-browserify',
      'http': 'stream-http',
      'https': 'https-browserify',
      'crypto': 'crypto-browserify',
      'buffer': 'buffer',
      'process': 'process/browser',
      'util': 'util',
      'assert': 'assert',
      'url': 'url',
      'zlib': 'browserify-zlib',
      'path': 'path-browserify',
      'fs': '',
      'net': '',
      'tls': '',
    },
  },
  define: {
    'process.env': {},
    global: 'globalThis',
  },
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis',
      },
    },
  },
  build: {
    commonjsOptions: {
      include: [/node_modules/],
    },
  },
})
