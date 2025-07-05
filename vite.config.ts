import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from "path";

const root = path.resolve(__dirname, "src");

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": root,
    },
  }
})
