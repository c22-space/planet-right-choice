import { defineMiddleware } from 'astro:middleware'

export const onRequest = defineMiddleware(async (context, next) => {
  const { pathname } = context.url
  const { cookies } = context

  // Admin routes guard
  if (pathname.startsWith('/admin') && pathname !== '/admin/login') {
    const cfJwt = context.request.headers.get('Cf-Access-Jwt-Assertion')
    const adminToken = cookies.get('admin_token')?.value

    if (!cfJwt && !adminToken) {
      return context.redirect('/admin/login')
    }

    if (adminToken) {
      const valid = await verifyAdminJwt(
        adminToken,
        (context.locals as Record<string, unknown>)['runtime']
          ? String(((context.locals as Record<string, unknown>)['runtime'] as Record<string, unknown>)['env']?.['ADMIN_JWT_SECRET'] ?? '')
          : '',
      )
      if (!valid) {
        cookies.delete('admin_token', { path: '/' })
        return context.redirect('/admin/login')
      }
    }
  }

  // User dashboard guard
  if (pathname.startsWith('/dashboard')) {
    const userToken = cookies.get('user_token')?.value
    if (!userToken) {
      return context.redirect('/?auth=required')
    }
  }

  return next()
})

async function verifyAdminJwt(token: string, secret: string): Promise<boolean> {
  if (!secret) return false
  try {
    const [header, payload, sig] = token.split('.')
    if (!header || !payload || !sig) return false

    const signingInput = `${header}.${payload}`
    const encoder = new TextEncoder()
    const key = await crypto.subtle.importKey(
      'raw',
      encoder.encode(secret),
      { name: 'HMAC', hash: 'SHA-256' },
      false,
      ['verify'],
    )
    const sigBytes = Uint8Array.from(
      atob(sig.replace(/-/g, '+').replace(/_/g, '/')),
      (c) => c.charCodeAt(0),
    )
    const valid = await crypto.subtle.verify(
      'HMAC',
      key,
      sigBytes,
      encoder.encode(signingInput),
    )
    if (!valid) return false

    const payloadData = JSON.parse(atob(payload)) as { exp?: number }
    if (payloadData.exp && payloadData.exp < Math.floor(Date.now() / 1000)) return false
    return true
  } catch {
    return false
  }
}
