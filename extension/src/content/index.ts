import type { BackgroundToContent } from '@better-cart/fp-types'
import { scanFpTags, extractPageSignals } from './scanner.js'
import { injectBanner } from './banner.js'
import { sendToBackground, newSessionId } from '../shared/messaging.js'
import { getStorage } from '../shared/storage.js'

async function main(): Promise<void> {
  const showBanner = await getStorage('showBanner')
  if (!showBanner) return

  const sessionId = newSessionId()
  const domain = window.location.hostname

  // 1. Try fp: tags
  const fpProduct = scanFpTags()
  const signals = await extractPageSignals(sessionId)

  if (fpProduct) {
    sendToBackground({
      type: 'FP_DETECTED',
      product: fpProduct,
      signals,
    })
  } else {
    sendToBackground({
      type: 'ESTIMATE_REQUESTED',
      signals,
    })
  }

  // 2. Listen for the service worker's response
  chrome.runtime.onMessage.addListener((msg: BackgroundToContent) => {
    if (msg.type === 'ALTERNATIVES_READY') {
      injectBanner({
        baseline: msg.baseline,
        alternatives: msg.alternatives,
        domain,
        sessionId,
      })
    }
  })
}

main().catch(console.error)
