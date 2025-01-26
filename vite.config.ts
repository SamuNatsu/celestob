import { defineConfig } from 'vite';
import path from 'node:path';

// Plugins
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';

// https://vite.dev/config/
export default defineConfig({
  build: {
    outDir: 'www'
  },
  plugins: [tailwindcss(), vue()],
  resolve: {
    alias: {
      '@': path.resolve('src-frontend')
    }
  },
  server: {
    proxy: {
      '/api': {
        changeOrigin: true,
        target: 'http://localhost:3000'
      }
    }
  }
});
