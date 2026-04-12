use crate::middleware::auth::admin_required;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let page: u32 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("pageSize").and_then(|v| v.parse().ok()).unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;
    let domain = params.get("domain").cloned();

    let db = ctx.env.d1("DB")?;

    // Summary grouped by domain
    let summary: Vec<serde_json::Value> = db
        .prepare(
            "SELECT domain,
                    COUNT(*) as detection_count,
                    MAX(detected_at) as last_detected,
                    COUNT(DISTINCT fp_version) as versions_seen
             FROM fp_detections
             GROUP BY domain
             ORDER BY detection_count DESC
             LIMIT ?1 OFFSET ?2",
        )
        .bind(&[(page_size as f64).into(), (offset as f64).into()])?
        .all()
        .await?
        .results()?;

    let total = db
        .prepare("SELECT COUNT(DISTINCT domain) as n FROM fp_detections")
        .first::<serde_json::Value>(None)
        .await?
        .and_then(|v| v["n"].as_i64())
        .unwrap_or(0);

    // Recent raw detections (optionally filtered by domain)
    let recent = if let Some(ref d) = domain {
        db.prepare(
            "SELECT id, domain, product_name, co2e_kg, co2e_scope, fp_version, raw_tags, detected_at
             FROM fp_detections WHERE domain = ?1
             ORDER BY detected_at DESC LIMIT 20",
        )
        .bind(&[d.clone().into()])?
        .all()
        .await?
        .results::<serde_json::Value>()?
    } else {
        db.prepare(
            "SELECT id, domain, product_name, co2e_kg, fp_version, detected_at
             FROM fp_detections ORDER BY detected_at DESC LIMIT 20",
        )
        .all()
        .await?
        .results::<serde_json::Value>()?
    };

    Response::from_json(&json!({
        "summary": summary,
        "recent": recent,
        "total": total,
        "page": page,
        "pageSize": page_size,
    }))
}
