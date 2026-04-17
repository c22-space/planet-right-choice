import type { FpProduct, PageSignals } from '@better-cart/fp-types'
import { hashUrl } from '../shared/messaging.js'

const FP_META_SELECTOR = 'meta[property^="fp:"]'

/** Scan the current document for fp: meta tags. Returns null if none found. */
export function scanFpTags(): FpProduct | null {
  const metas = document.head.querySelectorAll<HTMLMetaElement>(FP_META_SELECTOR)
  if (metas.length === 0) return null

  const tags: Record<string, string> = {}
  metas.forEach((meta) => {
    const property = meta.getAttribute('property')
    const content = meta.getAttribute('content')
    if (property && content !== null) tags[property] = content
  })

  const product = tags['fp:product']
  const co2eRaw = tags['fp:co2e']
  if (!product || !co2eRaw) return null

  const co2eValue = parseFloat(co2eRaw)
  if (isNaN(co2eValue) || co2eValue < 0) return null

  const unit = (tags['fp:co2e:unit'] ?? 'kg').toLowerCase()
  const co2eKg =
    unit === 'g' ? co2eValue / 1000 : unit === 't' ? co2eValue * 1000 : co2eValue

  const scope = (tags['fp:scope'] ?? 'lifecycle') as FpProduct['scope']

  return {
    product,
    co2eKg,
    scope,
    ...(tags['fp:certifier'] !== undefined ? { certifier: tags['fp:certifier'] } : {}),
    ...(tags['fp:version'] !== undefined ? { fpVersion: tags['fp:version'] } : {}),
    rawTags: { ...tags },
  }
}

/** Extract product signals from the page DOM for the estimation protocol. */
export async function extractPageSignals(sessionId: string): Promise<PageSignals> {
  const url = window.location.href
  const domain = window.location.hostname
  const pageUrlHash = await hashUrl(url)

  return {
    productName: extractProductName(),
    brand: extractBrand(),
    categoryBreadcrumb: extractBreadcrumb(),
    amazonCategory: extractAmazonCategory(),
    weightKg: extractWeight(),
    materialHints: extractMaterials(),
    originCountry: extractOriginCountry(),
    priceUsd: extractPrice(),
    asin: extractAsin(),
    domain,
    pageUrlHash,
    sessionId,
  }
}

// ─── Extractors ──────────────────────────────────────────────────────────────

function extractProductName(): string | null {
  // Schema.org Product
  const ld = extractSchemaOrg()
  if (ld?.name) return String(ld.name)

  // Amazon
  const amzTitle = document.getElementById('productTitle')
  if (amzTitle) return amzTitle.textContent?.trim() ?? null

  // Open Graph
  const ogTitle = document
    .querySelector('meta[property="og:title"]')
    ?.getAttribute('content')
  if (ogTitle) return ogTitle

  // Page title fallback
  return document.title || null
}

function extractBrand(): string | null {
  const ld = extractSchemaOrg()
  if (ld?.brand) {
    const b = ld.brand
    if (typeof b === 'string') return b
    if (typeof b === 'object' && b !== null && 'name' in b)
      return String((b as Record<string, unknown>)['name'])
  }

  const amzBrand = document.querySelector('#bylineInfo, [data-feature-name="bylineInfo"]')
  if (amzBrand) {
    const text = amzBrand.textContent?.trim() ?? ''
    const match = /(?:by|Brand:)\s+(.+)/i.exec(text)
    if (match?.[1]) return match[1].trim()
  }

  return null
}

function extractBreadcrumb(): string[] {
  // Schema.org BreadcrumbList
  const scripts = document.querySelectorAll<HTMLScriptElement>(
    'script[type="application/ld+json"]',
  )
  for (const script of scripts) {
    try {
      const data = JSON.parse(script.textContent ?? '')
      if (data['@type'] === 'BreadcrumbList' && Array.isArray(data.itemListElement)) {
        return data.itemListElement
          .map((item: Record<string, unknown>) => {
            const name = item['name'] ?? (item['item'] as Record<string, unknown>)?.['name']
            return typeof name === 'string' ? name : null
          })
          .filter(Boolean) as string[]
      }
    } catch {
      // ignore malformed JSON-LD
    }
  }

  // Amazon breadcrumb
  const amzCrumbs = document.querySelectorAll('#wayfinding-breadcrumbs_container a, .a-breadcrumb a')
  if (amzCrumbs.length > 0) {
    return Array.from(amzCrumbs)
      .map((el) => el.textContent?.trim() ?? '')
      .filter(Boolean)
  }

  return []
}

function extractAmazonCategory(): string | null {
  const node = document.querySelector(
    '#nav-subnav [data-category], [data-csa-c-slot-id="nav-subnav"]',
  )
  return node?.getAttribute('data-category') ?? null
}

function extractWeight(): number | null {
  // Look in spec tables for weight
  const specPatterns = [
    /item\s*weight[:\s]+([0-9.]+)\s*(kg|g|lb|lbs|oz)/i,
    /shipping\s*weight[:\s]+([0-9.]+)\s*(kg|g|lb|lbs|oz)/i,
    /weight[:\s]+([0-9.]+)\s*(kg|g|lb|lbs|oz)/i,
  ]

  const textContent = [
    document.querySelector('#productDetails_techSpec_section_1')?.textContent ?? '',
    document.querySelector('#detailBullets_feature_div')?.textContent ?? '',
    document.querySelector('.product-specs')?.textContent ?? '',
    document.querySelector('[class*="spec"]')?.textContent ?? '',
  ].join(' ')

  for (const pattern of specPatterns) {
    const match = pattern.exec(textContent)
    if (match?.[1] && match?.[2]) {
      const value = parseFloat(match[1])
      const unit = match[2].toLowerCase()
      if (unit === 'g') return value / 1000
      if (unit === 'lb' || unit === 'lbs') return value * 0.453592
      if (unit === 'oz') return value * 0.0283495
      return value // kg
    }
  }

  // Schema.org weight
  const ld = extractSchemaOrg()
  if (ld?.weight) {
    const w = ld.weight
    if (typeof w === 'object' && w !== null && 'value' in w) {
      return parseFloat(String((w as Record<string, unknown>)['value']))
    }
  }

  return null
}

function extractMaterials(): string[] {
  const materialKeywords = [
    'cotton', 'organic cotton', 'polyester', 'recycled polyester', 'nylon',
    'wool', 'merino', 'linen', 'silk', 'leather', 'vegan leather',
    'aluminium', 'aluminum', 'recycled aluminium', 'steel', 'stainless steel',
    'plastic', 'abs', 'polycarbonate', 'glass', 'recycled pet', 'rpet',
    'bamboo', 'wood', 'oak', 'pine', 'plywood', 'mdf',
    'rubber', 'silicone', 'carbon fibre', 'titanium',
  ]

  const text = [
    document.querySelector('#feature-bullets')?.textContent ?? '',
    document.querySelector('.product-description')?.textContent ?? '',
    document.querySelector('[id*="material"]')?.textContent ?? '',
    document.querySelector('[class*="material"]')?.textContent ?? '',
    document.querySelector('[id*="fabric"]')?.textContent ?? '',
  ]
    .join(' ')
    .toLowerCase()

  return materialKeywords.filter((kw) => text.includes(kw))
}

function extractOriginCountry(): string | null {
  const patterns = [
    /made\s+in\s+([A-Za-z\s]+?)(?:[\.,\n]|$)/i,
    /country\s+of\s+origin[:\s]+([A-Za-z\s]+?)(?:[\.,\n]|$)/i,
    /manufactured\s+in\s+([A-Za-z\s]+?)(?:[\.,\n]|$)/i,
  ]

  const text = [
    document.querySelector('#detailBullets_feature_div')?.textContent ?? '',
    document.querySelector('#productDetails_techSpec_section_1')?.textContent ?? '',
  ].join(' ')

  for (const pattern of patterns) {
    const match = pattern.exec(text)
    if (match?.[1]) {
      return countryNameToIso(match[1].trim()) ?? match[1].trim()
    }
  }

  return null
}

function extractPrice(): number | null {
  // Schema.org
  const ld = extractSchemaOrg()
  if (ld?.offers) {
    const offers = Array.isArray(ld.offers) ? ld.offers[0] : ld.offers
    const price = (offers as Record<string, unknown>)?.['price']
    if (price !== undefined) return parseFloat(String(price))
  }

  // Amazon
  const amzPrice = document.querySelector('.a-price .a-offscreen')?.textContent
  if (amzPrice) {
    const match = /[\d,]+\.?\d*/.exec(amzPrice.replace(/[^0-9.,]/g, ''))
    if (match) return parseFloat(match[0].replace(',', ''))
  }

  return null
}

function extractAsin(): string | null {
  const match = /\/dp\/([A-Z0-9]{10})/.exec(window.location.href)
  return match?.[1] ?? null
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

let _cachedSchemaOrg: Record<string, unknown> | null | undefined = undefined

function extractSchemaOrg(): Record<string, unknown> | null {
  if (_cachedSchemaOrg !== undefined) return _cachedSchemaOrg

  const scripts = document.querySelectorAll<HTMLScriptElement>(
    'script[type="application/ld+json"]',
  )
  for (const script of scripts) {
    try {
      const data = JSON.parse(script.textContent ?? '') as Record<string, unknown>
      const type = data['@type']
      if (type === 'Product' || (Array.isArray(type) && type.includes('Product'))) {
        _cachedSchemaOrg = data
        return data
      }
    } catch {
      // ignore
    }
  }

  _cachedSchemaOrg = null
  return null
}

const COUNTRY_MAP: Record<string, string> = {
  'china': 'CN', 'bangladesh': 'BD', 'india': 'IN', 'vietnam': 'VN',
  'taiwan': 'TW', 'south korea': 'KR', 'japan': 'JP', 'germany': 'DE',
  'france': 'FR', 'italy': 'IT', 'spain': 'ES', 'netherlands': 'NL',
  'united kingdom': 'GB', 'uk': 'GB', 'united states': 'US', 'usa': 'US',
  'canada': 'CA', 'portugal': 'PT', 'turkey': 'TR', 'indonesia': 'ID',
  'cambodia': 'KH', 'ethiopia': 'ET',
}

function countryNameToIso(name: string): string | null {
  return COUNTRY_MAP[name.toLowerCase()] ?? null
}
