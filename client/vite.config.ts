import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      stream: path.resolve(__dirname, 'node_modules/stream-browserify'),
      crypto: path.resolve(__dirname, 'node_modules/crypto-browserify'),
      buffer: path.resolve(__dirname, 'node_modules/buffer'),
      util: path.resolve(__dirname, 'node_modules/util'),
      assert: path.resolve(__dirname, 'node_modules/assert'),
      http: path.resolve(__dirname, 'node_modules/stream-http'),
      https: path.resolve(__dirname, 'node_modules/https-browserify'),
      os: path.resolve(__dirname, 'node_modules/os-browserify'),
      url: path.resolve(__dirname, 'node_modules/url'),
    },
  },
  define: {
    'process.env': {},
    global: 'globalThis',
  },
  optimizeDeps: {
    include: ['buffer', 'crypto-browserify', 'stream-browserify'],
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
