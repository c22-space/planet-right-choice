use crate::db::models::AffiliateRule;
use crate::middleware::auth::admin_required;
use serde::Deserialize;
use serde_json::json;
use worker::{Cache, Request, Response, Result, RouteContext};

const RULES_CACHE_KEY: &str = "https://cache.internal/v1/affiliate/rules";

#[derive(Deserialize)]
struct RuleBody {
    source_asin: String,
    target_asin: Option<String>,
    affiliate_tag: Option<String>,
    reason: Option<String>,
    priority: Option<i64>,
}

pub async fn list_rules(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let include_inactive = params.get("includeInactive").map(|v| v == "true").unwrap_or(false);
    let page: u32 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("pageSize").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset = (page - 1) * page_size;

    let where_clause = if include_inactive { "" } else { "WHERE is_active = 1" };

    let db = ctx.env.d1("DB")?;
    let rules: Vec<AffiliateRule> = db
        .prepare(format!(
            "SELECT id, source_asin, target_asin, affiliate_tag, reason, is_active, priority
             FROM affiliate_rules {where_clause}
             ORDER BY priority DESC, created_at DESC
             LIMIT ?1 OFFSET ?2"
        ))
        .bind(&[(page_size as f64).into(), (offset as f64).into()])?
        .all()
        .await?
        .results()?;

    let total = db
        .prepare(format!("SELECT COUNT(*) as n FROM affiliate_rules {where_clause}"))
        .first::<serde_json::Value>(None)
        .await?
        .and_then(|v| v["n"].as_i64())
        .unwrap_or(0);

    Response::from_json(&json!({ "rules": rules, "total": total, "page": page }))
}

pub async fn create_rule(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let body: RuleBody = req.json().await?;
    let default_tag = ctx.env.var("AFFILIATE_TAG")?.to_string();
    let db = ctx.env.d1("DB")?;

    let result = db
        .prepare(
            "INSERT INTO affiliate_rules (source_asin, target_asin, affiliate_tag, reason, priority)
             VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
        )
        .bind(&[
            body.source_asin.into(),
            body.target_asin.unwrap_or_default().into(),
            body.affiliate_tag.unwrap_or(default_tag).into(),
            body.reason.unwrap_or_default().into(),
            (body.priority.unwrap_or(0) as f64).into(),
        ])?
        .first::<serde_json::Value>(None)
        .await?;

    // Invalidate the public rules cache
    let cache = Cache::default();
    let _ = cache.delete(RULES_CACHE_KEY, false).await;

    let id = result.and_then(|v| v["id"].as_i64()).unwrap_or(0);
    Response::from_json(&json!({ "id": id }))
}

pub async fn update_rule(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let id: i64 = ctx.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);
    let body: RuleBody = req.json().await?;
    let default_tag = ctx.env.var("AFFILIATE_TAG")?.to_string();
    let db = ctx.env.d1("DB")?;

    db.prepare(
        "UPDATE affiliate_rules SET source_asin=?1, target_asin=?2, affiliate_tag=?3, reason=?4,
                                    priority=?5, updated_at=datetime('now')
         WHERE id=?6",
    )
    .bind(&[
        body.source_asin.into(),
        body.target_asin.unwrap_or_default().into(),
        body.affiliate_tag.unwrap_or(default_tag).into(),
        body.reason.unwrap_or_default().into(),
        (body.priority.unwrap_or(0) as f64).into(),
        id.into(),
    ])?
    .run()
    .await?;

    let cache = Cache::default();
    let _ = cache.delete(RULES_CACHE_KEY, false).await;

    Response::from_json(&json!({ "ok": true }))
}

pub async fn delete_rule(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let id: i64 = ctx.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);
    let db = ctx.env.d1("DB")?;

    db.prepare(
        "UPDATE affiliate_rules SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
    )
    .bind(&[id.into()])?
    .run()
    .await?;

    let cache = Cache::default();
    let _ = cache.delete(RULES_CACHE_KEY, false).await;

    Response::from_json(&json!({ "ok": true }))
}

pub async fn stats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let days: u32 = params.get("days").and_then(|v| v.parse().ok()).unwrap_or(30);

    let db = ctx.env.d1("DB")?;

    let daily: Vec<serde_json::Value> = db
        .prepare(
            "SELECT date(clicked_at) as date, COUNT(*) as clicks
             FROM affiliate_clicks
             WHERE clicked_at >= datetime('now', ?1)
             GROUP BY date(clicked_at)
             ORDER BY date ASC",
        )
        .bind(&[format!("-{days} days").into()])?
        .all()
        .await?
        .results()?;

    let top_asins: Vec<serde_json::Value> = db
        .prepare(
            "SELECT source_asin, COUNT(*) as clicks
             FROM affiliate_clicks
             WHERE clicked_at >= datetime('now', ?1)
             GROUP BY source_asin
             ORDER BY clicks DESC LIMIT 10",
        )
        .bind(&[format!("-{days} days").into()])?
        .all()
        .await?
        .results()?;

    let revenue: Vec<serde_json::Value> = db
        .prepare(
            "SELECT period_year, period_month,
                    SUM(clicks) as total_clicks,
                    SUM(orders) as total_orders,
                    SUM(gross_usd) as total_gross_usd,
                    SUM(net_usd) as total_net_usd
             FROM affiliate_revenue
             GROUP BY period_year, period_month
             ORDER BY period_year DESC, period_month DESC LIMIT 12",
        )
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({
        "daily": daily,
        "topAsins": top_asins,
        "revenue": revenue,
    }))
}
