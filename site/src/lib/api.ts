// Server-side API client — calls the API worker via service binding or HTTP.

type AstroEnv = {
  API_WORKER?: { fetch: typeof fetch }
  API_BASE?: string
}

function getBase(env?: AstroEnv): string {
  if (env?.API_BASE) return env.API_BASE
  return 'http://localhost:8787'
}

export async function apiGet<T>(
  path: string,
  env?: AstroEnv,
  headers?: Record<string, string>,
): Promise<T> {
  const base = getBase(env)
  const res = await fetch(`${base}${path}`, {
    headers: { 'Content-Type': 'application/json', ...headers },
  })
  if (!res.ok) throw new Error(`API error ${res.status}: ${path}`)
  return res.json() as Promise<T>
}

export async function apiPost<T>(
  path: string,
  body: unknown,
  env?: AstroEnv,
  headers?: Record<string, string>,
): Promise<T> {
  const base = getBase(env)
  const res = await fetch(`${base}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...headers },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`API error ${res.status}: ${path}`)
  return res.json() as Promise<T>
}

export async function apiPut<T>(
  path: string,
  body: unknown,
  env?: AstroEnv,
  headers?: Record<string, string>,
): Promise<T> {
  const base = getBase(env)
  const res = await fetch(`${base}${path}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json', ...headers },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`API error ${res.status}: ${path}`)
  return res.json() as Promise<T>
}

export async function apiDelete<T>(
  path: string,
  env?: AstroEnv,
  headers?: Record<string, string>,
): Promise<T> {
  const base = getBase(env)
  const res = await fetch(`${base}${path}`, {
    method: 'DELETE',
    headers: { 'Content-Type': 'application/json', ...headers },
  })
  if (!res.ok) throw new Error(`API error ${res.status}: ${path}`)
  return res.json() as Promise<T>
}
