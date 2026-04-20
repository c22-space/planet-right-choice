import type { PageStatus } from '@planet-right-choice/fp-types'
import { getMultiple, setStorage } from '../shared/storage.js'

async function main(): Promise<void> {
  const [tabs] = await Promise.all([
    chrome.tabs.query({ active: true, currentWindow: true }),
  ])
  const tab = tabs[0]
  const tabId = tab?.id

  // View switching
  const mainView = document.getElementById('main-view')!
  const settingsView = document.getElementById('settings-view')!
  document.getElementById('open-settings')!.addEventListener('click', () => {
    mainView.classList.remove('active')
    settingsView.classList.add('active')
  })
  document.getElementById('close-settings')!.addEventListener('click', () => {
    settingsView.classList.remove('active')
    mainView.classList.add('active')
  })

  const { showBanner, dataSharing, apiKey } = await getMultiple(['showBanner', 'dataSharing', 'apiKey'])

  // Settings toggles
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

  // API key
  const apiKeyInput = document.getElementById('api-key-input') as HTMLInputElement
  const saveApiKeyBtn = document.getElementById('save-api-key') as HTMLButtonElement
  const apiKeyStatus = document.getElementById('api-key-status') as HTMLParagraphElement
  if (apiKey) apiKeyInput.value = apiKey
  saveApiKeyBtn.addEventListener('click', async () => {
    await setStorage('apiKey', apiKeyInput.value.trim())
    apiKeyStatus.textContent = 'Saved.'
    apiKeyStatus.style.display = 'block'
    setTimeout(() => { apiKeyStatus.style.display = 'none' }, 2000)
  })

  // Poll storage until the service worker writes a result (up to 8s)
  if (tabId) {
    let attempts = 0
    const poll = () => {
      chrome.storage.local.get('pageStatuses', (result) => {
        const statuses = (result['pageStatuses'] as Record<number, PageStatus>) ?? {}
        const status = statuses[tabId] ?? null
        if (status?.scanned || attempts >= 8) {
          renderStatus(status)
        } else {
          attempts++
          setTimeout(poll, 1000)
        }
      })
    }
    poll()
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
