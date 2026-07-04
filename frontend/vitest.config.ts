import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  test: {
    // Pure util tests run in Node; bump to 'jsdom' if DOM is needed later.
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
})
