// ─── Footprint Protocol types ────────────────────────────────────────────────

export type Co2eScope =
  | 'lifecycle'
  | 'cradle-to-gate'
  | 'use-phase'
  | 'end-of-life'

export type Co2eSource = 'fp_tag' | 'estimated' | 'manual' | 'certified'

export interface FpProduct {
  product: string
  co2eKg: number
  scope: Co2eScope
  certifier?: string
  fpVersion?: string
  rawTags: Record<string, string>
}

// ─── Page signal extraction ──────────────────────────────────────────────────

export interface Material {
  name: string
  pct?: number // 0–100
}

export interface PageSignals {
  productName: string | null
  brand: string | null
  categoryBreadcrumb: string[]
  amazonCategory: string | null
  weightKg: number | null
  materialHints: string[]
  originCountry: string | null // ISO 3166-1 alpha-2
  priceUsd: number | null
  asin: string | null
  domain: string
  pageUrlHash: string // SHA-256 of the page URL — never the raw URL
  sessionId: string
}

// ─── Estimation protocol ─────────────────────────────────────────────────────

export type EstimationTier = 1 | 2 | 3

export interface EstimationResult {
  estimatedCo2eKg: number
  confidence: number // 0–1
  tier: EstimationTier
  methodVersion: string
  categorySlug: string | null
  signals: PageSignals
}

// ─── Product catalogue ───────────────────────────────────────────────────────

export interface Category {
  id: number
  slug: string
  name: string
  parentId: number | null
  avgCo2eKg: number | null
  avgCo2eScope: Co2eScope | null
  factorSource: string | null
}

export interface CatalogueProduct {
  id: number
  name: string
  brand: string
  categoryId: number
  categorySlug?: string
  asin: string | null
  url: string | null
  imageUrl: string | null
  description: string | null
  co2eKg: number | null
  co2eScope: Co2eScope | null
  co2eSource: Co2eSource | null
  co2eConfidence: number | null
  certifications: string[]
  materials: Material[]
  weightKg: number | null
  originCountry: string | null
  isActive: boolean
}

// ─── Affiliate ────────────────────────────────────────────────────────────────

export interface AffiliateRule {
  id: number
  sourceAsin: string
  targetAsin: string | null
  affiliateTag: string
  reason: string | null
  isActive: boolean
  priority: number
}

// ─── Scoring ──────────────────────────────────────────────────────────────────

export interface ProductScore {
  product: CatalogueProduct
  co2eKg: number
  vsCategory: number // ratio — 1.0 = average, <1.0 = better than average
  percentile: number // 0–100, higher = greener
}

export interface ScoredAlternative extends ProductScore {
  savingKg: number // CO2e saved vs the baseline product
}

// ─── Extension messaging ─────────────────────────────────────────────────────

export type ContentToBackground =
  | { type: 'FP_DETECTED'; product: FpProduct; signals: PageSignals }
  | { type: 'ESTIMATE_REQUESTED'; signals: PageSignals }
  | { type: 'GET_PAGE_STATUS' }

export type BackgroundToContent =
  | {
      type: 'ALTERNATIVES_READY'
      baseline: { co2eKg: number; source: Co2eSource; tier?: EstimationTier }
      alternatives: ScoredAlternative[]
    }
  | { type: 'PAGE_STATUS'; status: PageStatus }

export interface PageStatus {
  scanned: boolean
  fpDetected: boolean
  estimationTier: EstimationTier | null
  alternativeCount: number
  co2eKg: number | null
  source: Co2eSource | null
}

// ─── API response shapes ─────────────────────────────────────────────────────

export interface ApiError {
  error: string
  code?: string
}

export interface PaginatedResponse<T> {
  data: T[]
  page: number
  pageSize: number
  total: number
}
