import type {
  ContentToBackground,
  BackgroundToContent,
  PageStatus,
  ScoredAlternative,
  Co2eSource,
  EstimationTier,
} from '@better-cart/fp-types'

export type { ContentToBackground, BackgroundToContent }

export function sendToBackground(msg: ContentToBackground): void {
  chrome.runtime.sendMessage(msg)
}

export function sendToTab(tabId: number, msg: BackgroundToContent): void {
  chrome.tabs.sendMessage(tabId, msg)
}

export function onBackgroundMessage(
  handler: (msg: BackgroundToContent) => void,
): void {
  chrome.runtime.onMessage.addListener((msg: BackgroundToContent) => {
    handler(msg)
  })
}

// Generate a random session ID — not linked to any user identity
export function newSessionId(): string {
  const arr = new Uint8Array(16)
  crypto.getRandomValues(arr)
  return Array.from(arr, (b) => b.toString(16).padStart(2, '0')).join('')
}

// SHA-256 of a URL string — never store or transmit raw URLs
export async function hashUrl(url: string): Promise<string> {
  const buf = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode(url),
  )
  return Array.from(new Uint8Array(buf), (b) =>
    b.toString(16).padStart(2, '0'),
  ).join('')
}
