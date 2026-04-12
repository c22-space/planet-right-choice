use crate::db::models::AffiliateRule;
use crate::middleware::auth::api_key_required;
use serde::Deserialize;
use serde_json::json;
use worker::{Cache, Request, Response, Result, RouteContext};

const RULES_CACHE_KEY: &str = "https://cache.internal/v1/affiliate/rules";

pub async fn get_rules(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    // Check Cloudflare Cache API first
    let cache = Cache::default();
    if let Some(cached) = cache.get(RULES_CACHE_KEY, false).await? {
        return Ok(cached);
    }

    let db = ctx.env.d1("DB")?;
    let rules: Vec<AffiliateRule> = db
        .prepare(
            "SELECT id, source_asin, target_asin, affiliate_tag, reason, is_active, priority
             FROM affiliate_rules WHERE is_active = 1 ORDER BY priority DESC",
        )
        .all()
        .await?
        .results()?;

    let mut resp = Response::from_json(&json!({ "rules": rules }))?;
    resp.headers_mut()
        .set("Cache-Control", "public, max-age=43200")?; // 12h

    // Store in cache
    cache.put(RULES_CACHE_KEY, resp.cloned()?).await?;

    Ok(resp)
}

pub async fn rewrite(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let asin = url
        .query_pairs()
        .find(|(k, _)| k == "asin")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default();

    if asin.is_empty() {
        return Response::error("asin required", 400);
    }

    let db = ctx.env.d1("DB")?;
    let rule = db
        .prepare(
            "SELECT source_asin, target_asin, affiliate_tag FROM affiliate_rules
             WHERE source_asin = ?1 AND is_active = 1 ORDER BY priority DESC LIMIT 1",
        )
        .bind(&[asin.clone().into()])?
        .first::<AffiliateRule>(None)
        .await?;

    let affiliate_tag = ctx.env.var("AFFILIATE_TAG")?.to_string();

    match rule {
        Some(r) => Response::from_json(&json!({
            "sourceAsin": r.source_asin,
            "targetAsin": r.target_asin,
            "affiliateTag": r.affiliate_tag,
        })),
        None => Response::from_json(&json!({
            "sourceAsin": asin,
            "targetAsin": null,
            "affiliateTag": affiliate_tag,
        })),
    }
}

#[derive(Deserialize)]
struct ClickBody {
    source_asin: String,
    target_asin: Option<String>,
    session_id: String,
    rule_id: Option<i64>,
}

pub async fn record_click(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let body: ClickBody = req.json().await?;
    let country = req
        .headers()
        .get("CF-IPCountry")?
        .unwrap_or_else(|| "XX".into());

    let db = ctx.env.d1("DB")?;
    db.prepare(
        "INSERT INTO affiliate_clicks (rule_id, source_asin, target_asin, session_id, country_code)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&[
        body.rule_id.map(|id| id.into()).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.source_asin.into(),
        body.target_asin.unwrap_or_default().into(),
        body.session_id.into(),
        country.into(),
    ])?
    .run()
    .await?;

    Response::from_json(&json!({ "ok": true }))
}
