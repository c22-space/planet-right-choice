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
    let tier = params.get("tier").and_then(|v| v.parse::<i64>().ok());

    let db = ctx.env.d1("DB")?;

    let where_clause = if let Some(t) = tier {
        format!("WHERE e.tier = {t}")
    } else {
        String::new()
    };

    let total = db
        .prepare(&format!("SELECT COUNT(*) as n FROM estimations e {where_clause}"))
        .first::<serde_json::Value>(None)
        .await?
        .and_then(|v| v["n"].as_i64())
        .unwrap_or(0);

    let rows: Vec<serde_json::Value> = db
        .prepare(&format!(
            "SELECT e.id, e.domain, e.product_name, e.estimated_co2e_kg,
                    e.confidence, e.tier, e.method_version, e.created_at,
                    c.slug as category_slug, c.name as category_name
             FROM estimations e
             LEFT JOIN categories c ON e.category_id = c.id
             {where_clause}
             ORDER BY e.created_at DESC
             LIMIT ?1 OFFSET ?2"
        ))
        .bind(&[(page_size as f64).into(), (offset as f64).into()])?
        .all()
        .await?
        .results()?;

    // Confidence breakdown by tier
    let tier_stats: Vec<serde_json::Value> = db
        .prepare(
            "SELECT tier, COUNT(*) as count, AVG(confidence) as avg_confidence
             FROM estimations GROUP BY tier ORDER BY tier",
        )
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({
        "data": rows,
        "tierStats": tier_stats,
        "total": total,
        "page": page,
        "pageSize": page_size,
    }))
}
