import { fetchAffiliateRules } from './api-client.js'

export async function syncRules(): Promise<void> {
  try {
    const rules = await fetchAffiliateRules()
    await chrome.storage.local.set({
      affiliateRules: rules,
      rulesSyncedAt: Date.now(),
    })
    console.log(`[Better Cart] Synced ${rules.length} affiliate rules`)
  } catch (err) {
    console.warn('[Better Cart] Rules sync failed:', err)
  }
}

export function setupRulesSync(): void {
  // Schedule a 12-hour periodic sync via chrome.alarms
  chrome.alarms.create('rules-sync', { periodInMinutes: 720 })
  chrome.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === 'rules-sync') {
      syncRules()
    }
  })

  // Run immediately on install / startup if stale
  chrome.storage.local.get('rulesSyncedAt', (result) => {
    const syncedAt = (result['rulesSyncedAt'] as number | undefined) ?? 0
    const stale = Date.now() - syncedAt > 12 * 60 * 60 * 1000
    if (stale) {
      syncRules()
    }
  })
}
