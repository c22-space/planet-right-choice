import { defineConfig } from 'astro/config'
import cloudflare from '@astrojs/cloudflare'
import tailwind from '@astrojs/tailwind'

export default defineConfig({
  adapter: cloudflare({ mode: 'directory' }),
  output: 'server',
  integrations: [tailwind()],
})
