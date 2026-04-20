use crate::db::models::{resolve_image, Product};
use crate::middleware::auth::api_key_required;
use crate::services::estimation::{estimate as run_estimate, EstimationInput};
use fp_parser::scan_html;
use serde::Deserialize;
use serde_json::json;
use worker::{Fetch, Request, Response, Result, RouteContext};

#[derive(Deserialize)]
struct ParseBody {
    html_snippet: Option<String>,
    tags: Option<std::collections::HashMap<String, String>>,
    session_id: String,
    domain: String,
    page_url_hash: String,
    #[allow(dead_code)]
    product_name: Option<String>,
}

#[derive(Deserialize)]
struct EstimateBody {
    signals: EstimationInput,
}

pub async fn parse(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }
    if req.headers().get("content-length")?.map(|v| v.parse::<usize>().unwrap_or(0)).unwrap_or(0) > 2_000_000 {
        return Response::error("Payload too large", 413);
    }
    let body: ParseBody = req.json().await?;

    // Try from raw tags first, then HTML snippet
    let product = if let Some(tags) = body.tags {
        fp_parser::parse_fp_tags(&tags).ok()
    } else if let Some(html) = body.html_snippet {
        scan_html(&html)
    } else {
        None
    };

    let Some(product) = product else {
        return Response::error("No valid fp: tags found", 422);
    };

    let db = ctx.env.d1("DB")?;
    db.prepare(
        "INSERT INTO fp_detections (session_id, domain, page_url_hash, product_name, co2e_kg, co2e_scope, fp_version, raw_tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&[
        body.session_id.into(),
        body.domain.into(),
        body.page_url_hash.into(),
        product.product.clone().into(),
        product.co2e_kg.into(),
        serde_json::to_string(&product.scope).unwrap_or_default().into(),
        product.fp_version.clone().unwrap_or_default().into(),
        serde_json::to_string(&product.raw_tags).unwrap_or_default().into(),
    ])?
    .run()
    .await?;

    Response::from_json(&json!({ "product": product }))
}

pub async fn estimate(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }
    if req.headers().get("content-length")?.map(|v| v.parse::<usize>().unwrap_or(0)).unwrap_or(0) > 2_000_000 {
        return Response::error("Payload too large", 413);
    }
    let body: EstimateBody = req.json().await?;

    let session_id = body.signals.session_id.clone();
    let domain = body.signals.domain.clone();
    let page_url_hash = body.signals.page_url_hash.clone();
    let product_name = body.signals.product_name.clone().unwrap_or_default();

    let result = run_estimate(body.signals);

    let db = ctx.env.d1("DB")?;
    let _ = db
        .prepare(
            "INSERT INTO estimations (session_id, domain, page_url_hash, product_name, signals, estimated_co2e_kg, confidence, tier, method_version)
             VALUES (?1, ?2, ?3, ?4, '{}', ?5, ?6, ?7, ?8)",
        )
        .bind(&[
            session_id.into(),
            domain.into(),
            page_url_hash.into(),
            product_name.into(),
            result.estimated_co2e_kg.into(),
            result.confidence.into(),
            (result.tier as i64 as f64).into(),
            result.method_version.clone().into(),
        ])?
        .run()
        .await;

    Response::from_json(&json!({ "result": result }))
}

/// Extract simple signals from any product page HTML using meta tags / JSON-LD.
fn extract_signals(html: &str, url: &str) -> EstimationInput {
    let meta = |prop: &str| -> Option<String> {
        let patterns = [
            format!(r#"property="{prop}"[^>]+content="([^"]+)""#),
            format!(r#"content="([^"]+)"[^>]+property="{prop}""#),
            format!(r#"name="{prop}"[^>]+content="([^"]+)""#),
            format!(r#"content="([^"]+)"[^>]+name="{prop}""#),
        ];
        for pat in &patterns {
            if let Ok(re) = regex_lite::Regex::new(pat) {
                if let Some(cap) = re.captures(html) {
                    return Some(cap[1].trim().to_string());
                }
            }
        }
        None
    };

    let product_name = meta("og:title")
        .or_else(|| meta("twitter:title"))
        .or_else(|| {
            // fallback: <title>...</title>
            let re = regex_lite::Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
            re.captures(html).map(|c| c[1].trim().to_string())
        });

    let brand = meta("og:brand")
        .or_else(|| meta("product:brand"))
        .or_else(|| meta("og:site_name"));

    let price_usd = meta("product:price:amount")
        .or_else(|| meta("og:price:amount"))
        .and_then(|s| s.parse::<f64>().ok());

    let domain = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    let description = meta("og:description").or_else(|| meta("description"));

    // Try JSON-LD BreadcrumbList first, then fall back to description as a hint
    let category_breadcrumb = {
        let mut crumbs: Vec<String> = vec![];

        if let Ok(re) = regex_lite::Regex::new(
            r#"(?s)"@type"\s*:\s*"BreadcrumbList".*?"itemListElement"\s*:\s*\[([^\]]+)\]"#,
        ) {
            if let Some(cap) = re.captures(html) {
                if let Ok(name_re) = regex_lite::Regex::new(r#""name"\s*:\s*"([^"]+)""#) {
                    crumbs = name_re
                        .captures_iter(&cap[1])
                        .map(|c| c[1].to_string())
                        .collect();
                }
            }
        }

        if crumbs.is_empty() {
            if let Some(desc) = description.as_deref() {
                crumbs = vec![desc.split_whitespace().take(10).collect::<Vec<_>>().join(" ")];
            }
        }

        crumbs
    };

    EstimationInput {
        product_name,
        brand,
        category_breadcrumb,
        amazon_category: None,
        weight_kg: None,
        material_hints: vec![],
        origin_country: None,
        price_usd,
        asin: None,
        domain,
        page_url_hash: String::new(),
        session_id: String::new(),
    }
}

#[derive(Deserialize)]
struct FromUrlBody {
    url: String,
}

fn is_safe_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else { return false };
    if parsed.scheme() != "https" { return false }
    let host = parsed.host_str().unwrap_or("");
    if host.is_empty() { return false }
    let blocked = ["localhost", "127.", "10.", "192.168.", "169.254.", "0.", "::1", "fc", "fd"];
    if blocked.iter().any(|b| host.starts_with(b)) { return false }
    // Block 172.16.0.0/12
    if host.starts_with("172.") {
        if let Some(second) = host.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()) {
            if (16..=31).contains(&second) { return false }
        }
    }
    true
}

/// Public: accepts any product URL, checks catalogue then estimates.
pub async fn from_url(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: FromUrlBody = req.json().await?;
    let url = body.url.trim().to_string();

    if !is_safe_url(&url) {
        return Response::error("Invalid or disallowed URL", 422);
    }

    let db = ctx.env.d1("DB")?;

    // 1. Check catalogue by exact URL match
    let mut rows: Vec<Product> = db
        .prepare("SELECT * FROM products WHERE url = ?1 AND is_active = 1 LIMIT 1")
        .bind(&[url.clone().into()])?
        .all()
        .await?
        .results()?;

    if let Some(p) = rows.pop().map(resolve_image) {
        return Response::from_json(&json!({ "source": "catalogue", "product": p }));
    }

    // 2. Fetch the page and extract signals
    let fetch_req = Request::new(&url, worker::Method::Get)?;
    let mut page_resp = Fetch::Request(fetch_req).send().await
        .map_err(|_| worker::Error::RustError("Could not fetch the product page".into()))?;

    if page_resp.status_code() >= 400 {
        return Response::error("Could not fetch the product page", 422);
    }

    let html = page_resp.text().await?;

    // Try fp: tags first
    if let Some(fp) = scan_html(&html) {
        return Response::from_json(&json!({
            "source": "fp_tags",
            "co2e_kg": fp.co2e_kg,
            "product_name": fp.product,
            "confidence": 1.0,
            "tier": 0,
        }));
    }

    // Fall back to estimation from meta signals
    let signals = extract_signals(&html, &url);
    let result = run_estimate(signals);

    Response::from_json(&json!({ "source": "estimated", "estimate": result }))
}

pub async fn get_product_footprint(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id: i64 = ctx
        .param("product_id")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| worker::Error::RustError("invalid product_id".into()))?;

    let db = ctx.env.d1("DB")?;
    let row = db
        .prepare(
            "SELECT id, name, brand, co2e_kg, co2e_scope, co2e_source, co2e_confidence
             FROM products WHERE id = ?1 AND is_active = 1",
        )
        .bind(&[id.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    match row {
        Some(v) => Response::from_json(&json!({ "product": v })),
        None => Response::error("Not found", 404),
    }
}
