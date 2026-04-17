import type { AffiliateRule, ScoredAlternative, EstimationResult } from '@planet-right-choice/fp-types'

const API_BASE = import.meta.env['VITE_API_BASE'] ?? 'https://api.rightchoice.c22.space'
const MAX_RETRIES = 3

async function fetchWithRetry(
  url: string,
  init: RequestInit,
  retries = MAX_RETRIES,
): Promise<Response> {
  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const res = await fetch(url, init)
      if (res.ok || res.status < 500) return res
    } catch {
      if (attempt === retries - 1) throw new Error(`Network error fetching ${url}`)
      await new Promise((r) => setTimeout(r, 2 ** attempt * 500))
    }
  }
  throw new Error(`All retries failed for ${url}`)
}

async function getApiKey(): Promise<string> {
  const result = await chrome.storage.local.get('apiKey')
  return (result['apiKey'] as string | undefined) ?? ''
}

function headers(apiKey: string): Record<string, string> {
  return { 'Content-Type': 'application/json', 'X-API-Key': apiKey }
}

export async function fetchAlternativesByAsin(asin: string): Promise<{
  alternatives: ScoredAlternative[]
  baseline: { asin: string; co2eKg: number } | null
}> {
  const key = await getApiKey()
  const res = await fetchWithRetry(
    `${API_BASE}/v1/alternatives/asin/${asin}`,
    { headers: headers(key) },
  )
  if (!res.ok) return { alternatives: [], baseline: null }
  return res.json()
}

export async function fetchAlternativesByCategory(
  categorySlug: string,
  baselineCo2eKg: number,
): Promise<ScoredAlternative[]> {
  const key = await getApiKey()
  const qs = new URLSearchParams({
    category: categorySlug,
    baselineCo2eKg: String(baselineCo2eKg),
    limit: '3',
  })
  const res = await fetchWithRetry(
    `${API_BASE}/v1/alternatives?${qs}`,
    { headers: headers(key) },
  )
  if (!res.ok) return []
  const data = await res.json() as { alternatives: ScoredAlternative[] }
  return data.alternatives
}

export async function postFpDetection(payload: {
  tags: Record<string, string>
  sessionId: string
  domain: string
  pageUrlHash: string
}): Promise<void> {
  const key = await getApiKey()
  await fetchWithRetry(`${API_BASE}/v1/footprint/parse`, {
    method: 'POST',
    headers: headers(key),
    body: JSON.stringify(payload),
  })
}

export async function postEstimate(payload: {
  signals: Record<string, unknown>
}): Promise<EstimationResult | null> {
  const key = await getApiKey()
  const res = await fetchWithRetry(`${API_BASE}/v1/footprint/estimate`, {
    method: 'POST',
    headers: headers(key),
    body: JSON.stringify(payload),
  })
  if (!res.ok) return null
  const data = await res.json() as { result: EstimationResult }
  return data.result
}

export async function fetchAffiliateRules(): Promise<AffiliateRule[]> {
  const key = await getApiKey()
  const res = await fetchWithRetry(`${API_BASE}/v1/affiliate/rules`, {
    headers: headers(key),
  })
  if (!res.ok) return []
  const data = await res.json() as { rules: AffiliateRule[] }
  return data.rules
}

export async function postAffiliateClick(payload: {
  sourceAsin: string
  targetAsin?: string
  sessionId: string
  ruleId?: number
}): Promise<void> {
  const key = await getApiKey()
  await fetchWithRetry(`${API_BASE}/v1/affiliate/click`, {
    method: 'POST',
    headers: headers(key),
    body: JSON.stringify({
      source_asin: payload.sourceAsin,
      target_asin: payload.targetAsin,
      session_id: payload.sessionId,
      rule_id: payload.ruleId,
    }),
  })
}

export async function postImpactEvent(payload: {
  sessionId: string
  domain: string
  baselineCo2eKg: number
  alternativeCo2eKg: number
  baselineSource: string
  baselineTier?: number
  alternativeId?: number
  categorySlug?: string
}): Promise<void> {
  const key = await getApiKey()
  await fetchWithRetry(`${API_BASE}/v1/impact/record`, {
    method: 'POST',
    headers: headers(key),
    body: JSON.stringify({
      session_id: payload.sessionId,
      domain: payload.domain,
      baseline_co2e_kg: payload.baselineCo2eKg,
      alternative_co2e_kg: payload.alternativeCo2eKg,
      baseline_source: payload.baselineSource,
      baseline_tier: payload.baselineTier,
      alternative_id: payload.alternativeId,
      category_slug: payload.categorySlug,
    }),
  }).catch(() => { /* fire and forget */ })
}
