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

    // Verify bcrypt hash — we use a simple comparison here;
    // in production use a proper bcrypt crate. For now we store
    // SHA-256 hex hashes seeded during setup.
    let input_hash = sha256_hex(body.password.as_bytes());
    if input_hash != stored_hash {
        return Response::error("Invalid credentials", 401);
    }

    // Update last login
    let user_id = user["id"].as_i64().unwrap_or(0);
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

fn sha256_hex(data: &[u8]) -> String {
    // Minimal SHA-256 implementation for password verification.
    // In production, replace with a bcrypt crate compiled to WASM.
    use std::num::Wrapping;

    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
        0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
        0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [Wrapping<u32>; 8] = [
        Wrapping(0x6a09e667u32), Wrapping(0xbb67ae85), Wrapping(0x3c6ef372), Wrapping(0xa54ff53a),
        Wrapping(0x510e527f), Wrapping(0x9b05688c), Wrapping(0x1f83d9ab), Wrapping(0x5be0cd19),
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [Wrapping(0u32); 64];
        for (i, b) in chunk.chunks(4).enumerate().take(16) {
            w[i] = Wrapping(u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
        }
        for i in 16..64 {
            let s0 = w[i-15].0.rotate_right(7) ^ w[i-15].0.rotate_right(18) ^ (w[i-15].0 >> 3);
            let s1 = w[i-2].0.rotate_right(17) ^ w[i-2].0.rotate_right(19) ^ (w[i-2].0 >> 10);
            w[i] = Wrapping(w[i-16].0.wrapping_add(s0).wrapping_add(w[i-7].0).wrapping_add(s1));
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.0.rotate_right(6) ^ e.0.rotate_right(11) ^ e.0.rotate_right(25);
            let ch = (e.0 & f.0) ^ (!e.0 & g.0);
            let t1 = Wrapping(hh.0.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i].0));
            let s0 = a.0.rotate_right(2) ^ a.0.rotate_right(13) ^ a.0.rotate_right(22);
            let maj = (a.0 & b.0) ^ (a.0 & c.0) ^ (b.0 & c.0);
            let t2 = Wrapping(s0.wrapping_add(maj));
            hh = g; g = f; f = e; e = Wrapping(d.0.wrapping_add(t1.0));
            d = c; c = b; b = a; a = Wrapping(t1.0.wrapping_add(t2.0));
        }
        h[0] += a; h[1] += b; h[2] += c; h[3] += d;
        h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    }

    h.iter().map(|v| format!("{:08x}", v.0)).collect()
}
