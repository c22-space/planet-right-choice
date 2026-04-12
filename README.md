# Better Cart

A browser extension + Cloudflare web app that reads [Footprint Protocol](https://footprint.c22.foundation/) tags from product pages and suggests greener alternatives — so you can spend your money on brands that actually give a shit.

## What it does

1. **Reads `fp:` meta tags** — detects Footprint Protocol metadata on any product page (CO₂e, lifecycle scope, certifier, etc.)
2. **Estimates when tags are absent** — runs a three-tier standardised protocol (material-based → weight-scaled → spend-based EEIO) when no tags are found
3. **Scores the product** — compares the carbon footprint against category averages
4. **Suggests alternatives** — surfaces lower-footprint products from its catalogue
5. **Tracks real impact** — records CO₂e saved each time a user clicks a greener alternative, surfaced as a live counter on the homepage
6. **Rewrites Amazon links** — replaces Amazon product URLs with affiliate links, with revenue funding the project

## Architecture

```
planet-right/
├── Cargo.toml                 # Rust workspace (api + shared crates)
├── pnpm-workspace.yaml        # pnpm workspace (extension + site + TS packages)
│
├── packages/
│   ├── fp-types/              # Shared TypeScript interfaces
│   ├── fp-parser/             # Rust: fp: tag parser (native in API, WASM for extension)
│   └── estimation-engine/     # Rust: 3-tier estimation protocol
│
├── extension/                 # MV3 browser extension (Chrome + Firefox)
│   ├── src/content/           # Scanner, shadow DOM banner (impact tracking on click)
│   ├── src/background/        # Service worker, affiliate URL rewriting, rules sync
│   └── src/popup/             # Settings, page status
│
├── api/                       # Cloudflare Worker — Rust (workers-rs)
│   └── src/
│       ├── routes/            # footprint, alternatives, affiliate, catalogue, impact, admin
│       ├── middleware/        # API key auth, admin JWT
│       └── migrations/        # 0001–0005 D1 SQL migrations + category seed
│
└── site/                      # Astro 5 + Cloudflare Pages
    └── src/pages/
        ├── index.astro        # Marketing + live impact counter + seller adoption section
        ├── how-it-works.astro
        ├── install.astro
        ├── dashboard/         # User impact dashboard
        └── admin/             # Admin portal (overview, catalogue, estimations,
                               #   fp: detections, affiliate, impact)
```

## Footprint Protocol integration

Products that publish `fp:` meta tags get a **verified score** — buyers see your real data, not an estimate. Verified products rank higher in Better Cart's alternatives suggestions.

```html
<meta property="fp:product"   content="Fairphone 5" />
<meta property="fp:co2e"      content="23.6" />
<meta property="fp:co2e:unit" content="kg" />
<meta property="fp:scope"     content="lifecycle" />
```

→ [Read the Footprint Protocol](https://footprint.c22.foundation/)

## Estimation protocol (when no fp: tags found)

| Tier | Method | Confidence | Requires |
|------|--------|-----------|---------|
| 1 | Material-based hybrid (ecoinvent 3.9 + Higg MSI) | 65–88% | weight + material hints |
| 2 | Weight × category intensity | 40–64% | weight only |
| 3 | Price × USEEIO v2.1 spend factor | 15–39% | price or category |

Every estimation stores its `method_version` so results can be re-scored when the methodology improves.

## Impact tracking

Every time a user clicks a recommended alternative in the banner, Better Cart records:
- Baseline CO₂e (from fp: tag or estimate)
- Alternative CO₂e
- CO₂e saved = baseline − alternative

Aggregated totals are shown on the homepage and in the admin `/admin/impact` page, broken down by category, source quality, and day. This gives both shoppers and sellers a concrete, auditable measure of the platform's real-world effect.

## Stack

| Layer | Tech |
|---|---|
| Extension | MV3 (Chrome/Firefox), Vanilla TypeScript |
| Web app | Astro 5, Cloudflare Pages |
| API | Cloudflare Workers (Rust, `workers-rs`), D1, R2 |
| fp: parsing | Rust `fp-parser` crate — native in API, compiled to WASM for extension |
| Estimation engine | Rust `estimation-engine` crate — shared across API + extension |
| Admin portal | Astro SSR pages at `/admin/*`, JWT cookie or Cloudflare Access |
| CI | GitHub Actions — typecheck, Rust check + test, D1 migration validation |

## Getting started

```bash
# Install all workspace dependencies
pnpm install

# Start the API worker locally
pnpm dev:api

# Start the site (marketing + admin)
pnpm dev:site

# Build the extension (Chrome)
pnpm dev:ext

# Build the extension (Firefox)
cd extension && pnpm build:firefox

# Apply D1 migrations locally
cd api && pnpm db:migrate:local
```

## Admin portal

The admin portal at `/admin/` requires a signed-in admin user. Sections:

| Page | What it shows |
|------|--------------|
| `/admin/` | Overview: detections, estimations, clicks |
| `/admin/catalogue` | Add / edit / soft-delete products |
| `/admin/footprints` | Estimation results with tier & confidence breakdown |
| `/admin/detections` | fp: tag detection adoption by domain |
| `/admin/affiliate` | ASIN override rules + click & revenue stats |
| `/admin/impact` | CO₂e saved, greener clicks, daily chart, breakdown by category |

## Related

- [Footprint Protocol](https://footprint.c22.foundation/) — the open standard for product carbon metadata
