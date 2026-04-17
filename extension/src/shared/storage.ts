import type { AffiliateRule, PageStatus } from '@planet-right-choice/fp-types'

export interface ExtensionStorage {
  affiliateRules: AffiliateRule[]
  rulesSyncedAt: number
  apiKey: string
  showBanner: boolean
  dataSharing: boolean
  pageStatuses: Record<string, PageStatus> // keyed by tabId
}

const DEFAULTS: ExtensionStorage = {
  affiliateRules: [],
  rulesSyncedAt: 0,
  apiKey: '',
  showBanner: true,
  dataSharing: true,
  pageStatuses: {},
}

export async function getStorage<K extends keyof ExtensionStorage>(
  key: K,
): Promise<ExtensionStorage[K]> {
  const result = await chrome.storage.local.get(key)
  const value = result[key]
  return (value !== undefined ? value : DEFAULTS[key]) as ExtensionStorage[K]
}

export async function setStorage<K extends keyof ExtensionStorage>(
  key: K,
  value: ExtensionStorage[K],
): Promise<void> {
  await chrome.storage.local.set({ [key]: value })
}

export async function getMultiple<K extends keyof ExtensionStorage>(
  keys: K[],
): Promise<Pick<ExtensionStorage, K>> {
  const result = await chrome.storage.local.get(keys)
  const out = {} as Pick<ExtensionStorage, K>
  for (const key of keys) {
    out[key] = (result[key] !== undefined ? result[key] : DEFAULTS[key]) as ExtensionStorage[K]
  }
  return out
}
