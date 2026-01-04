import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    outDir: 'dist/js-bundle',
    lib: {
      entry: 'js/editor.js',
      name: 'HoldsmithEditor',
      fileName: 'editor',
      formats: ['es']
    },
    rollupOptions: {
      external: [],
      output: {
        // Don't hash the filename for easier reference
        entryFileNames: 'editor.js'
      }
    }
  }
});
