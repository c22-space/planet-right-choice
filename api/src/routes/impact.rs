use crate::middleware::auth::{admin_required, api_key_required};
use serde::Deserialize;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

/// Public: aggregate impact stats (no auth — used by the homepage counter)
pub async fn public_stats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;

    let totals = db
        .prepare(
            "SELECT COUNT(*) as total_events,
                    COALESCE(SUM(saving_co2e_kg), 0) as total_saving_co2e_kg,
                    COUNT(DISTINCT session_id) as unique_sessions
             FROM impact_events",
        )
        .first::<serde_json::Value>(None)
        .await?;

    let by_category: Vec<serde_json::Value> = db
        .prepare(
            "SELECT category_slug,
                    COUNT(*) as events,
                    SUM(saving_co2e_kg) as total_saving_kg
             FROM impact_events
             WHERE category_slug IS NOT NULL
             GROUP BY category_slug
             ORDER BY total_saving_kg DESC
             LIMIT 10",
        )
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({
        "totals": totals,
        "byCategory": by_category,
    }))
}

#[derive(Deserialize)]
struct ImpactBody {
    session_id: String,
    domain: String,
    baseline_co2e_kg: f64,
    alternative_co2e_kg: f64,
    baseline_source: String,
    baseline_tier: Option<i64>,
    alternative_id: Option<i64>,
    category_slug: Option<String>,
}

/// Record an impact event when a user clicks a recommended alternative.
pub async fn record(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let body: ImpactBody = req.json().await?;

    if body.alternative_co2e_kg >= body.baseline_co2e_kg {
        return Response::error("alternative_co2e_kg must be less than baseline_co2e_kg", 400);
    }

    let country = req
        .headers()
        .get("CF-IPCountry")?
        .unwrap_or_else(|| "XX".into());

    let db = ctx.env.d1("DB")?;
    db.prepare(
        "INSERT INTO impact_events
           (session_id, domain, baseline_co2e_kg, alternative_co2e_kg,
            baseline_source, baseline_tier, alternative_id, category_slug, country_code)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(&[
        body.session_id.into(),
        body.domain.into(),
        body.baseline_co2e_kg.into(),
        body.alternative_co2e_kg.into(),
        body.baseline_source.into(),
        body.baseline_tier
            .map(|v| worker::wasm_bindgen::JsValue::from_f64(v as f64))
            .unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.alternative_id
            .map(|v| worker::wasm_bindgen::JsValue::from_f64(v as f64))
            .unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.category_slug.unwrap_or_default().into(),
        country.into(),
    ])?
    .run()
    .await?;

    Response::from_json(&json!({ "ok": true }))
}

/// Admin: detailed impact breakdown
pub async fn admin_stats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let days: u32 = params.get("days").and_then(|v| v.parse().ok()).unwrap_or(30);

    let db = ctx.env.d1("DB")?;

    let totals = db
        .prepare(
            "SELECT COUNT(*) as total_events,
                    COALESCE(SUM(saving_co2e_kg), 0) as total_saving_co2e_kg,
                    COUNT(DISTINCT session_id) as unique_sessions,
                    AVG(saving_co2e_kg) as avg_saving_kg
             FROM impact_events
             WHERE created_at >= datetime('now', ?1)",
        )
        .bind(&[format!("-{days} days").into()])?
        .first::<serde_json::Value>(None)
        .await?;

    let daily: Vec<serde_json::Value> = db
        .prepare(
            "SELECT date(created_at) as date,
                    COUNT(*) as events,
                    SUM(saving_co2e_kg) as saving_kg
             FROM impact_events
             WHERE created_at >= datetime('now', ?1)
             GROUP BY date(created_at)
             ORDER BY date ASC",
        )
        .bind(&[format!("-{days} days").into()])?
        .all()
        .await?
        .results()?;

    let by_source: Vec<serde_json::Value> = db
        .prepare(
            "SELECT baseline_source, COUNT(*) as events, SUM(saving_co2e_kg) as saving_kg
             FROM impact_events
             WHERE created_at >= datetime('now', ?1)
             GROUP BY baseline_source",
        )
        .bind(&[format!("-{days} days").into()])?
        .all()
        .await?
        .results()?;

    let by_category: Vec<serde_json::Value> = db
        .prepare(
            "SELECT category_slug, COUNT(*) as events, SUM(saving_co2e_kg) as saving_kg
             FROM impact_events
             WHERE created_at >= datetime('now', ?1) AND category_slug IS NOT NULL
             GROUP BY category_slug
             ORDER BY saving_kg DESC LIMIT 10",
        )
        .bind(&[format!("-{days} days").into()])?
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({
        "totals": totals,
        "daily": daily,
        "bySource": by_source,
        "byCategory": by_category,
    }))
}
