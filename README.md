# Better Cart

A browser extension + Tauri desktop app that reads [Footprint Protocol](https://footprint.earth) tags from product pages and suggests greener alternatives — so you can spend your money on brands that actually give a shit.

## What it does

1. **Reads `fp:` meta tags** — detects Footprint Protocol metadata on any product page (CO₂e, lifecycle scope, certifier, etc.)
2. **Scores the product** — compares the carbon footprint against category averages
3. **Suggests alternatives** — surfaces lower-footprint products from its catalogue
4. **Rewrites Amazon links** — replaces Amazon product URLs with affiliate links, with revenue funding the project

## Architecture

```
better-cart/
├── extension/          # Chrome/Firefox extension (MV3)
│   ├── content/        # Content script: reads fp: tags, injects UI
│   ├── background/     # Service worker: affiliate link rewriting, API calls
│   └── popup/          # Popup UI: settings, account
│
├── app/                # Tauri desktop app (wraps the Astro site)
│   └── src-tauri/
│
├── site/               # Astro marketing + dashboard site
│   └── src/
│
└── api/                # Backend (Cloudflare Workers + D1)
    ├── footprint/      # fp: tag parsing + product scoring
    ├── alternatives/   # Sustainable product catalogue + matching
    └── affiliate/      # Amazon affiliate link rewriting
```

## How the Footprint Protocol integration works

The extension content script scans for `<meta property="fp:*">` tags on any page:

```html
<!-- Detected automatically by Better Cart -->
<meta property="fp:product"   content="Fairphone 5" />
<meta property="fp:co2e"      content="23.6" />
<meta property="fp:co2e:unit" content="kg" />
<meta property="fp:scope"     content="lifecycle" />
```

If no `fp:` tags are found, the extension falls back to category-level heuristics based on page URL and product title.

## Affiliate link rewriting

When a user visits an Amazon product page, the background service worker:

1. Captures the ASIN from the URL
2. Checks the alternatives catalogue for a lower-footprint product
3. If a greener option exists — shows a suggestion banner
4. Rewrites the Amazon URL with the affiliate tag regardless (revenue funds the catalogue)

Override rules are stored in Cloudflare D1 and synced to the extension via a periodic background fetch.

## Stack

| Layer | Tech |
|---|---|
| Extension | MV3 (Chrome/Firefox), Vanilla TS |
| Desktop app | Tauri v2 |
| Site | Astro 5, Cloudflare Pages |
| API | Cloudflare Workers, D1, R2 |
| fp: parsing | Rust (`footprint-parser` crate) via WASM |

## Getting started

```bash
# Extension
cd extension && pnpm install && pnpm dev

# Site
cd site && pnpm install && pnpm dev

# Tauri app (requires Rust)
cd app && pnpm install && pnpm tauri dev

# API (requires Wrangler)
cd api && pnpm install && wrangler dev
```

## Related

- [Footprint Protocol](https://footprint.earth) — the open standard for product carbon metadata
- [footprint-parser](https://crates.io/crates/footprint-parser) — Rust crate for parsing `fp:` tags
