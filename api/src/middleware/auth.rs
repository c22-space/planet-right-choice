use hmac::{Hmac, Mac};
use sha2::Sha256;
use worker::{Env, Request, Response, Result};

type HmacSha256 = Hmac<Sha256>;

/// Verify the X-API-Key header against the API_SECRET binding.
pub fn verify_api_key(req: &Request, env: &Env) -> Result<bool> {
    let secret = env.secret("API_SECRET")?.to_string();
    let key = req.headers().get("X-API-Key")?.unwrap_or_default();
    Ok(!key.is_empty() && key == secret)
}

/// Verify an admin JWT from the Cookie header or Cf-Access-Jwt-Assertion.
pub fn verify_admin(req: &Request, env: &Env) -> Result<bool> {
    // Cloudflare Access sets this header when configured (production)
    if req.headers().get("Cf-Access-Jwt-Assertion")?.is_some() {
        return Ok(true);
    }

    // Accept Authorization: Bearer <token>
    let auth_header = req.headers().get("Authorization")?.unwrap_or_default();
    if let Some(bearer) = auth_header.strip_prefix("Bearer ") {
        let secret = env.secret("ADMIN_JWT_SECRET")?.to_string();
        return verify_jwt(bearer, &secret);
    }

    // Fall back to our own cookie-based JWT
    let cookie_header = req.headers().get("Cookie")?.unwrap_or_default();
    let token = extract_cookie_value(&cookie_header, "admin_token");

    let Some(token) = token else {
        return Ok(false);
    };

    let secret = env.secret("ADMIN_JWT_SECRET")?.to_string();
    verify_jwt(token, &secret)
}

pub fn admin_required(req: &Request, env: &Env) -> Result<Option<Response>> {
    if !verify_admin(req, env)? {
        return Ok(Some(Response::error("Unauthorized", 401)?));
    }
    Ok(None)
}

pub fn api_key_required(req: &Request, env: &Env) -> Result<Option<Response>> {
    if !verify_api_key(req, env)? {
        return Ok(Some(Response::error("Unauthorized", 401)?));
    }
    Ok(None)
}

fn extract_cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(name) {
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value);
            }
        }
    }
    None
}

pub fn sign_admin_jwt(payload_json: &str, secret: &str) -> Result<String> {
    let header = base64_url_encode(r#"{"alg":"HS256","typ":"JWT"}"#.as_bytes());
    let exp = worker::js_sys::Date::now() as u64 / 1000 + 28800; // 8 hours
    let payload_with_exp = {
        let mut v: serde_json::Value = serde_json::from_str(payload_json)
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        v["exp"] = serde_json::Value::Number(exp.into());
        serde_json::to_string(&v)
            .map_err(|e| worker::Error::RustError(e.to_string()))?
    };
    let encoded_payload = base64_url_encode(payload_with_exp.as_bytes());
    let signing_input = format!("{header}.{encoded_payload}");

    let sig = hmac_sha256(signing_input.as_bytes(), secret.as_bytes())?;
    let sig_b64 = base64_url_encode(&sig);

    Ok(format!("{signing_input}.{sig_b64}"))
}

fn verify_jwt(token: &str, secret: &str) -> Result<bool> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Ok(false);
    }

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected_sig = hmac_sha256(signing_input.as_bytes(), secret.as_bytes())?;
    let expected_b64 = base64_url_encode(&expected_sig);

    if expected_b64 != parts[2] {
        return Ok(false);
    }

    // Check expiry
    let payload_bytes = base64_url_decode(parts[1])?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    if let Some(exp) = payload.get("exp").and_then(|v| v.as_u64()) {
        let now = worker::js_sys::Date::now() as u64 / 1000;
        if exp < now {
            return Ok(false);
        }
    }

    Ok(true)
}

fn hmac_sha256(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn base64_url_encode(data: &[u8]) -> String {
    let b64 = {
        let mut s = String::new();
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let iter = data.chunks(3);
        for chunk in iter {
            let (b0, b1, b2) = match chunk.len() {
                1 => (chunk[0], 0, 0),
                2 => (chunk[0], chunk[1], 0),
                _ => (chunk[0], chunk[1], chunk[2]),
            };
            let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
            s.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
            s.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
            if chunk.len() > 1 {
                s.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
            } else {
                s.push('=');
            }
            if chunk.len() > 2 {
                s.push(TABLE[(n & 0x3f) as usize] as char);
            } else {
                s.push('=');
            }
        }
        s
    };
    b64.replace('+', "-").replace('/', "_").replace('=', "")
}

fn base64_url_decode(s: &str) -> Result<Vec<u8>> {
    let padded = {
        let mut s = s.replace('-', "+").replace('_', "/");
        while !s.len().is_multiple_of(4) {
            s.push('=');
        }
        s
    };
    // Simple base64 decode
    let mut out = Vec::new();
    const DECODE: [i8; 256] = {
        let mut t = [-1i8; 256];
        let mut i = 0u8;
        while i < 64 {
            let c = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"[i as usize];
            t[c as usize] = i as i8;
            i += 1;
        }
        t
    };
    for chunk in padded.as_bytes().chunks(4) {
        if chunk.len() < 4 { break; }
        let (a, b, c, d) = (
            DECODE[chunk[0] as usize],
            DECODE[chunk[1] as usize],
            if chunk[2] == b'=' { 0 } else { DECODE[chunk[2] as usize] },
            if chunk[3] == b'=' { 0 } else { DECODE[chunk[3] as usize] },
        );
        if a < 0 || b < 0 { break; }
        out.push(((a << 2) | (b >> 4)) as u8);
        if chunk[2] != b'=' { out.push(((b << 4) | (c >> 2)) as u8); }
        if chunk[3] != b'=' { out.push(((c << 6) | d) as u8); }
    }
    Ok(out)
}
