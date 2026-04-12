import { defineConfig } from 'vite'
import { resolve } from 'path'
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'fs'

// Plugin: copy manifest.json and static assets to dist/
function extensionPlugin(target: 'chrome' | 'firefox' = 'chrome') {
  return {
    name: 'extension-assets',
    closeBundle() {
      // Merge manifest
      const base = JSON.parse(readFileSync(resolve(__dirname, 'manifest.json'), 'utf-8'))
      if (target === 'firefox') {
        const overrides = JSON.parse(
          readFileSync(resolve(__dirname, 'manifest.firefox.json'), 'utf-8'),
        )
        Object.assign(base, overrides)
      }
      writeFileSync(resolve(__dirname, 'dist/manifest.json'), JSON.stringify(base, null, 2))

      // Copy popup HTML into dist/popup/
      mkdirSync(resolve(__dirname, 'dist/popup'), { recursive: true })
      copyFileSync(
        resolve(__dirname, 'src/popup/index.html'),
        resolve(__dirname, 'dist/popup/index.html'),
      )
    },
  }
}

export default defineConfig({
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        'content/index': resolve(__dirname, 'src/content/index.ts'),
        'background/service-worker': resolve(__dirname, 'src/background/service-worker.ts'),
        'popup/popup': resolve(__dirname, 'src/popup/popup.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: 'chunks/[name]-[hash].js',
        assetFileNames: '[name][extname]',
        format: 'esm',
      },
    },
    target: 'es2022',
    sourcemap: true,
    minify: false, // keep readable for extension review
  },
  plugins: [extensionPlugin((process.env['BROWSER'] as 'chrome' | 'firefox') ?? 'chrome')],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
})
