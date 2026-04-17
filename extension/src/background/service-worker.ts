import type { ContentToBackground, ScoredAlternative } from '@planet-right-choice/fp-types'
import { setupRulesSync } from './rules-sync.js'
import {
  fetchAlternativesByAsin,
  fetchAlternativesByCategory,
  postFpDetection,
  postEstimate,
  postImpactEvent,
} from './api-client.js'
import { rewriteAmazonUrl, extractAsin, handleAffiliateClick } from './affiliate.js'
import { newSessionId } from '../shared/messaging.js'

// Session ID is ephemeral — created fresh each SW startup
const SESSION_ID = newSessionId()
const DEFAULT_AFFILIATE_TAG = 'bettercart-21'

// ── Lifecycle ────────────────────────────────────────────────────────────────

self.addEventListener('install', () => {
  ;(self as unknown as ServiceWorkerGlobalScope).skipWaiting()
})

self.addEventListener('activate', (event: Event) => {
  const e = event as ExtendableEvent
  e.waitUntil((self as unknown as ServiceWorkerGlobalScope).clients.claim())
  setupRulesSync()
})

// ── Message handler ──────────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener(
  (
    msg: ContentToBackground & { _tabId?: number },
    sender,
    _sendResponse,
  ) => {
    const tabId = sender.tab?.id
    if (!tabId) return

    switch (msg.type) {
      case 'FP_DETECTED':
        handleFpDetected(tabId, msg).catch(console.error)
        break

      case 'ESTIMATE_REQUESTED':
        handleEstimateRequested(tabId, msg).catch(console.error)
        break

      case 'GET_PAGE_STATUS':
        chrome.storage.local.get('pageStatuses', (result) => {
          const statuses = (result['pageStatuses'] as Record<number, unknown>) ?? {}
          chrome.tabs.sendMessage(tabId, {
            type: 'PAGE_STATUS',
            status: statuses[tabId] ?? null,
          })
        })
        break
    }
  },
)

// ── Impact event relay ───────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener(
  (msg: { type: 'IMPACT_CLICK'; payload: Parameters<typeof postImpactEvent>[0] & { alternativeName?: string; productName?: string } }) => {
    if (msg.type === 'IMPACT_CLICK') {
      const p = msg.payload
      // Persist locally for the dashboard
      persistImpactEvent({
        domain: p.domain,
        baselineCo2eKg: p.baselineCo2eKg,
        alternativeCo2eKg: p.alternativeCo2eKg,
        savingKg: p.baselineCo2eKg - p.alternativeCo2eKg,
        categorySlug: p.categorySlug ?? null,
        productName: p.productName ?? null,
        alternativeName: p.alternativeName ?? null,
        ts: Date.now(),
      }).catch(() => {})
      // Send to API (fire-and-forget)
      postImpactEvent(p).catch(() => {})
    }
  },
)

async function persistImpactEvent(event: {
  domain: string
  baselineCo2eKg: number
  alternativeCo2eKg: number
  savingKg: number
  categorySlug: string | null
  productName: string | null
  alternativeName: string | null
  ts: number
}): Promise<void> {
  const result = await chrome.storage.local.get(['impactEvents'])
  const events: typeof event[] = (result['impactEvents'] as typeof event[] | undefined) ?? []
  events.unshift(event)
  // Keep last 200 events
  if (events.length > 200) events.length = 200
  await chrome.storage.local.set({ impactEvents: events })
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async function incrementScanCount(): Promise<void> {
  const result = await chrome.storage.local.get(['totalScans'])
  const n: number = ((result['totalScans'] as number | undefined) ?? 0) + 1
  await chrome.storage.local.set({ totalScans: n })
}

async function handleFpDetected(
  tabId: number,
  msg: Extract<ContentToBackground, { type: 'FP_DETECTED' }>,
): Promise<void> {
  const { product, signals } = msg

  incrementScanCount().catch(() => {})

  // Fire-and-forget: record the detection
  postFpDetection({
    tags: product.rawTags,
    sessionId: SESSION_ID,
    domain: signals.domain,
    pageUrlHash: signals.pageUrlHash,
  }).catch(() => {})

  // Fetch alternatives
  let alternatives: ScoredAlternative[] = []
  if (signals.asin) {
    const res = await fetchAlternativesByAsin(signals.asin)
    alternatives = res.alternatives
  } else {
    // Classify from breadcrumb
    const categorySlug = guessCategoryFromBreadcrumb(signals.categoryBreadcrumb)
    if (categorySlug) {
      alternatives = await fetchAlternativesByCategory(categorySlug, product.co2eKg)
    }
  }

  chrome.tabs.sendMessage(tabId, {
    type: 'ALTERNATIVES_READY',
    baseline: { co2eKg: product.co2eKg, source: 'fp_tag' as const },
    alternatives,
  })

  // Store page status
  await updatePageStatus(tabId, {
    scanned: true,
    fpDetected: true,
    estimationTier: null,
    alternativeCount: alternatives.length,
    co2eKg: product.co2eKg,
    source: 'fp_tag',
  })
}

async function handleEstimateRequested(
  tabId: number,
  msg: Extract<ContentToBackground, { type: 'ESTIMATE_REQUESTED' }>,
): Promise<void> {
  incrementScanCount().catch(() => {})
  const { signals } = msg

  const result = await postEstimate({
    signals: {
      ...signals,
      session_id: SESSION_ID,
    },
  })

  if (!result) {
    await updatePageStatus(tabId, {
      scanned: true,
      fpDetected: false,
      estimationTier: null,
      alternativeCount: 0,
      co2eKg: null,
      source: null,
    })
    return
  }

  const alternatives = await fetchAlternativesByCategory(
    result.categorySlug ?? 'general',
    result.estimatedCo2eKg,
  )

  chrome.tabs.sendMessage(tabId, {
    type: 'ALTERNATIVES_READY',
    baseline: {
      co2eKg: result.estimatedCo2eKg,
      source: 'estimated' as const,
      tier: result.tier,
    },
    alternatives,
  })

  await updatePageStatus(tabId, {
    scanned: true,
    fpDetected: false,
    estimationTier: result.tier,
    alternativeCount: alternatives.length,
    co2eKg: result.estimatedCo2eKg,
    source: 'estimated',
  })
}

// ── Amazon URL rewriting via webNavigation ───────────────────────────────────

chrome.webNavigation?.onBeforeNavigate?.addListener(async (details) => {
  if (!details.url.includes('amazon.')) return

  const asin = extractAsin(details.url)
  if (!asin) return

  const result = await chrome.storage.local.get(['affiliateRules'])
  const rules = (result['affiliateRules'] as import('@planet-right-choice/fp-types').AffiliateRule[]) ?? []

  const { rewritten, rule } = rewriteAmazonUrl(details.url, rules, DEFAULT_AFFILIATE_TAG)

  if (rewritten !== details.url) {
    // Record click
    handleAffiliateClick({
      sourceAsin: asin,
      ...(rule?.targetAsin ? { targetAsin: rule.targetAsin } : {}),
      sessionId: SESSION_ID,
      ...(rule?.id !== undefined ? { ruleId: rule.id } : {}),
    }).catch(() => {})

    if (details.tabId > 0) {
      chrome.tabs.update(details.tabId, { url: rewritten })
    }
  }
})

// ── Helpers ──────────────────────────────────────────────────────────────────

async function updatePageStatus(
  tabId: number,
  status: import('@planet-right-choice/fp-types').PageStatus,
): Promise<void> {
  const result = await chrome.storage.local.get('pageStatuses')
  const statuses = (result['pageStatuses'] as Record<number, unknown>) ?? {}
  statuses[tabId] = status
  await chrome.storage.local.set({ pageStatuses: statuses })
}

function guessCategoryFromBreadcrumb(breadcrumb: string[]): string | null {
  const text = breadcrumb.join(' ').toLowerCase()
  if (text.includes('phone') || text.includes('smartphone')) return 'electronics/smartphones'
  if (text.includes('laptop') || text.includes('computer')) return 'electronics/laptops'
  if (text.includes('t-shirt') || text.includes('tee')) return 'clothing/tshirts'
  if (text.includes('jeans') || text.includes('denim')) return 'clothing/jeans'
  if (text.includes('shoes') || text.includes('sneaker')) return 'clothing/shoes'
  if (text.includes('electronics')) return 'electronics/general'
  if (text.includes('clothing') || text.includes('apparel')) return 'clothing/general'
  return null
}
