import type { AffiliateRule } from '@planet-right-choice/fp-types'
import { postAffiliateClick } from './api-client.js'

const ASIN_REGEX = /\/dp\/([A-Z0-9]{10})/

export function extractAsin(url: string): string | null {
  const match = ASIN_REGEX.exec(url)
  return match?.[1] ?? null
}

export function rewriteAmazonUrl(
  url: string,
  rules: AffiliateRule[],
  defaultTag: string,
): { rewritten: string; rule: AffiliateRule | null } {
  const asin = extractAsin(url)
  if (!asin) return { rewritten: url, rule: null }

  const rule = rules
    .filter((r) => r.isActive && r.sourceAsin === asin)
    .sort((a, b) => b.priority - a.priority)[0] ?? null

  if (rule?.targetAsin) {
    // Redirect to a greener alternative
    const targetUrl = new URL(`https://www.amazon.com/dp/${rule.targetAsin}`)
    targetUrl.searchParams.set('tag', rule.affiliateTag)
    return { rewritten: targetUrl.toString(), rule }
  }

  // Just append our affiliate tag to the original URL
  const rewritten = new URL(url)
  rewritten.searchParams.set(
    'tag',
    rule?.affiliateTag ?? defaultTag,
  )
  return { rewritten: rewritten.toString(), rule }
}

export async function handleAffiliateClick(payload: {
  sourceAsin: string
  targetAsin?: string
  sessionId: string
  ruleId?: number
}): Promise<void> {
  await postAffiliateClick(payload)
}
