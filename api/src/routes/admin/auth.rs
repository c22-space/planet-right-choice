use crate::middleware::auth::sign_admin_jwt;
use serde::Deserialize;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
}

pub async fn login(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: LoginBody = req.json().await?;

    let db = ctx.env.d1("DB")?;
    let row = db
        .prepare("SELECT id, email, password_hash, role FROM admin_users WHERE email = ?1")
        .bind(&[body.email.clone().into()])?
        .first::<serde_json::Value>(None)
        .await?;

    let Some(user) = row else {
        return Response::error("Invalid credentials", 401);
    };

    let stored_hash = user["password_hash"].as_str().unwrap_or_default();
    let user_id = user["id"].as_i64().unwrap_or(0);

    let valid = if stored_hash.starts_with("pbkdf2$") {
        pbkdf2_verify(body.password.as_bytes(), stored_hash)
    } else {
        // Legacy SHA-256 path — constant-time comparison to avoid timing leaks
        constant_time_eq(sha256_hex(body.password.as_bytes()).as_bytes(), stored_hash.as_bytes())
    };

    if !valid {
        return Response::error("Invalid credentials", 401);
    }

    // Transparently upgrade legacy SHA-256 hashes to PBKDF2 on first login
    if !stored_hash.starts_with("pbkdf2$") {
        if let Ok(new_hash) = pbkdf2_hash(body.password.as_bytes()) {
            if let Ok(stmt) = db
                .prepare("UPDATE admin_users SET password_hash = ?1 WHERE id = ?2")
                .bind(&[new_hash.into(), user_id.into()])
            {
                let _ = stmt.run().await;
            }
        }
    }

    // Update last login
    db.prepare("UPDATE admin_users SET last_login_at = datetime('now') WHERE id = ?1")
        .bind(&[user_id.into()])?
        .run()
        .await?;

    let secret = ctx.env.secret("ADMIN_JWT_SECRET")?.to_string();
    let payload = serde_json::to_string(&json!({
        "sub": user["id"],
        "email": body.email,
        "role": user["role"],
    }))
    .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let token = sign_admin_jwt(&payload, &secret)?;

    let mut resp = Response::from_json(&json!({ "ok": true, "token": token }))?;
    resp.headers_mut().set(
        "Set-Cookie",
        &format!("admin_token={token}; HttpOnly; Secure; SameSite=Strict; Max-Age=28800; Path=/"),
    )?;

    Ok(resp)
}

const PBKDF2_ITERS: u32 = 600_000;

fn pbkdf2_hash(password: &[u8]) -> Result<String> {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    let mut dk = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, &salt, PBKDF2_ITERS, &mut dk);
    Ok(format!(
        "pbkdf2${}${}${}",
        PBKDF2_ITERS,
        hex_encode(&salt),
        hex_encode(&dk)
    ))
}

fn pbkdf2_verify(password: &[u8], stored: &str) -> bool {
    let parts: Vec<&str> = stored.splitn(4, '$').collect();
    if parts.len() != 4 || parts[0] != "pbkdf2" {
        return false;
    }
    let Ok(iters) = parts[1].parse::<u32>() else { return false };
    let Some(salt) = hex_decode(parts[2]) else { return false };
    let Some(expected_dk) = hex_decode(parts[3]) else { return false };
    let mut dk = vec![0u8; expected_dk.len()];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, &salt, iters, &mut dk);
    constant_time_eq(&dk, &expected_dk)
}

// Legacy path — kept only to verify old SHA-256 hashes during migration.
fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(data);
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 { return None; }
    s.as_bytes()
        .chunks(2)
        .map(|c| {
            let hi = (c[0] as char).to_digit(16)? as u8;
            let lo = (c[1] as char).to_digit(16)? as u8;
            Some((hi << 4) | lo)
        })
        .collect()
}
