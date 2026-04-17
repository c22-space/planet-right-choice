import type { PageStatus } from '@planet-right-choice/fp-types'
import { getMultiple, setStorage } from '../shared/storage.js'

async function main(): Promise<void> {
  const [tabs] = await Promise.all([
    chrome.tabs.query({ active: true, currentWindow: true }),
  ])
  const tab = tabs[0]
  const tabId = tab?.id

  const { showBanner, dataSharing } = await getMultiple(['showBanner', 'dataSharing'])

  // Render settings
  const bannerToggle = document.getElementById('toggle-banner') as HTMLInputElement
  bannerToggle.checked = showBanner
  bannerToggle.addEventListener('change', () => {
    setStorage('showBanner', bannerToggle.checked)
  })

  const sharingToggle = document.getElementById('toggle-sharing') as HTMLInputElement
  sharingToggle.checked = dataSharing
  sharingToggle.addEventListener('change', () => {
    setStorage('dataSharing', sharingToggle.checked)
  })

  // Fetch current page status
  if (tabId) {
    chrome.tabs.sendMessage(tabId, { type: 'GET_PAGE_STATUS' }, (status: PageStatus | null) => {
      renderStatus(status)
    })
  } else {
    renderStatus(null)
  }
}

function renderStatus(status: PageStatus | null): void {
  const statusEl = document.getElementById('page-status')!

  if (!status || !status.scanned) {
    statusEl.innerHTML = `<p class="muted">No product detected on this page.</p>`
    return
  }

  const sourceLabel = status.fpDetected
    ? '<span class="badge verified">✓ Verified</span>'
    : `<span class="badge estimated">≈ Estimated Tier ${status.estimationTier ?? '?'}</span>`

  statusEl.innerHTML = `
    <div class="score-row">
      <span class="co2">${status.co2eKg != null ? `${status.co2eKg.toFixed(1)} kg CO₂e` : '—'}</span>
      ${sourceLabel}
    </div>
    <p class="alts">${status.alternativeCount} greener alternative${status.alternativeCount !== 1 ? 's' : ''} found</p>
  `
}

main().catch(console.error)
