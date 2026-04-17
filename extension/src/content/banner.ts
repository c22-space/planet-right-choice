import type { ScoredAlternative, Co2eSource, EstimationTier } from '@planet-right-choice/fp-types'

const BANNER_ID = 'planet-right-choice-root'
const DISMISSED_KEY = 'dismissed'

interface BannerData {
  baseline: { co2eKg: number; source: Co2eSource; tier?: EstimationTier }
  alternatives: ScoredAlternative[]
  domain: string
  sessionId: string
}

let currentBannerData: BannerData | null = null

export function injectBanner(data: BannerData): void {
  // Avoid duplicates
  if (document.getElementById(BANNER_ID)) return

  // Check if dismissed for this ASIN
  const asin = new URL(window.location.href).pathname.match(/\/dp\/([A-Z0-9]{10})/)?.[1]
  if (asin) {
    const dismissed = sessionStorage.getItem(`${DISMISSED_KEY}:${asin}`)
    if (dismissed) return
  }

  currentBannerData = data
  const host = document.createElement('div')
  host.id = BANNER_ID
  const shadow = host.attachShadow({ mode: 'open' })

  shadow.innerHTML = `
    <style>
      :host { all: initial; }
      * { box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }

      .banner {
        position: fixed;
        bottom: 20px;
        right: 20px;
        z-index: 2147483647;
        width: 340px;
        max-height: 80vh;
        overflow-y: auto;
        border-radius: 16px;
        background: #fff;
        box-shadow: 0 8px 32px rgba(0,0,0,0.18), 0 2px 8px rgba(0,0,0,0.08);
        border: 1px solid #e5e7eb;
        animation: slideIn 0.25s ease-out;
      }

      @keyframes slideIn {
        from { transform: translateY(20px); opacity: 0; }
        to   { transform: translateY(0);   opacity: 1; }
      }

      .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 14px 16px 10px;
        border-bottom: 1px solid #f3f4f6;
      }

      .logo { font-size: 12px; font-weight: 700; color: #16a34a; letter-spacing: 0.05em; }
      .close-btn {
        background: none; border: none; cursor: pointer;
        font-size: 18px; color: #9ca3af; padding: 2px 6px; border-radius: 6px;
        line-height: 1;
      }
      .close-btn:hover { background: #f3f4f6; color: #374151; }

      .baseline {
        padding: 12px 16px;
        border-bottom: 1px solid #f3f4f6;
      }

      .co2-value {
        font-size: 26px;
        font-weight: 800;
        color: #111827;
        line-height: 1;
        margin-bottom: 2px;
      }

      .co2-label { font-size: 11px; color: #6b7280; }

      .source-badge {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 2px 8px;
        border-radius: 99px;
        font-size: 10px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        margin-top: 6px;
      }

      .source-verified { background: #dcfce7; color: #166534; }
      .source-estimated { background: #fef9c3; color: #854d0e; }

      .alternatives-label {
        padding: 10px 16px 6px;
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: #6b7280;
      }

      .alt-card {
        display: flex;
        align-items: flex-start;
        gap: 10px;
        padding: 10px 16px;
        cursor: pointer;
        transition: background 0.1s;
        border-radius: 0;
        text-decoration: none;
        color: inherit;
      }
      .alt-card:hover { background: #f9fafb; }

      .alt-img {
        width: 48px;
        height: 48px;
        object-fit: contain;
        border-radius: 8px;
        background: #f3f4f6;
        flex-shrink: 0;
      }

      .alt-info { flex: 1; min-width: 0; }
      .alt-name { font-size: 13px; font-weight: 600; color: #111827; margin-bottom: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      .alt-brand { font-size: 11px; color: #6b7280; margin-bottom: 4px; }
      .alt-savings {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 2px 6px;
        background: #dcfce7;
        color: #166534;
        border-radius: 6px;
        font-size: 10px;
        font-weight: 700;
      }

      .footer {
        padding: 8px 16px 12px;
        text-align: center;
      }
      .footer a { font-size: 10px; color: #9ca3af; text-decoration: none; }
      .footer a:hover { color: #6b7280; }
    </style>

    <div class="banner" role="dialog" aria-label="Right Choice — carbon footprint">
      <div class="header">
        <span class="logo">RIGHT CHOICE</span>
        <button class="close-btn" id="close-btn" aria-label="Dismiss">×</button>
      </div>

      <div class="baseline">
        <div class="co2-value" id="co2-value">—</div>
        <div class="co2-label">kg CO₂e · lifecycle</div>
        <div class="source-badge" id="source-badge">—</div>
      </div>

      <div id="alternatives-section"></div>

      <div class="footer">
        <a href="https://rightchoice.c22.space/how-it-works" target="_blank" rel="noopener">
          How we calculate this →
        </a>
      </div>
    </div>
  `

  document.body.appendChild(host)

  // Wire close button
  shadow.getElementById('close-btn')?.addEventListener('click', () => {
    host.remove()
    if (asin) sessionStorage.setItem(`${DISMISSED_KEY}:${asin}`, '1')
  })

  render(shadow, data)
}

function render(shadow: ShadowRoot, data: BannerData): void {
  const { baseline, alternatives } = data

  // Baseline value
  const co2El = shadow.getElementById('co2-value')!
  co2El.textContent = `${baseline.co2eKg.toFixed(1)} kg`

  // Source badge
  const badgeEl = shadow.getElementById('source-badge')!
  if (baseline.source === 'fp_tag' || baseline.source === 'certified') {
    badgeEl.className = 'source-badge source-verified'
    badgeEl.textContent = '✓ Verified'
  } else {
    badgeEl.className = 'source-badge source-estimated'
    const tier = baseline.tier ? ` Tier ${baseline.tier}` : ''
    badgeEl.textContent = `≈ Estimated${tier}`
  }

  // Alternatives
  const altSection = shadow.getElementById('alternatives-section')!
  if (alternatives.length > 0) {
    altSection.innerHTML = `
      <div class="alternatives-label">Greener alternatives</div>
      ${alternatives
        .map(
          (alt, idx) => `
        <a class="alt-card" href="${alt.product.url ?? '#'}" data-idx="${idx}" data-id="${alt.product.id}" target="_blank" rel="noopener">
          ${alt.product.imageUrl ? `<img class="alt-img" src="${alt.product.imageUrl}" alt="" loading="lazy" />` : '<div class="alt-img"></div>'}
          <div class="alt-info">
            <div class="alt-name">${escapeHtml(alt.product.name)}</div>
            <div class="alt-brand">${escapeHtml(alt.product.brand)}</div>
            <div class="alt-savings">↓ ${alt.savingKg.toFixed(1)} kg CO₂e saved</div>
          </div>
        </a>
      `,
        )
        .join('')}
    `

    // Wire impact tracking on alt click
    altSection.querySelectorAll('.alt-card').forEach((el) => {
      el.addEventListener('click', () => {
        const idx = parseInt(el.getAttribute('data-idx') ?? '0', 10)
        const alt = alternatives[idx]
        if (!alt || !currentBannerData) return

        chrome.runtime.sendMessage({
          type: 'IMPACT_CLICK',
          payload: {
            sessionId: currentBannerData.sessionId,
            domain: currentBannerData.domain,
            baselineCo2eKg: currentBannerData.baseline.co2eKg,
            alternativeCo2eKg: alt.co2eKg,
            baselineSource: currentBannerData.baseline.source,
            baselineTier: currentBannerData.baseline.tier,
            alternativeId: alt.product.id,
            categorySlug: alt.product.categorySlug,
            alternativeName: alt.product.name,
          },
        })
      })
    })
  }
}

function escapeHtml(str: string): string {
  return str.replace(/[&<>"']/g, (c) => {
    const map: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' }
    return map[c] ?? c
  })
}
