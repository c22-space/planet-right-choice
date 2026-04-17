use crate::services::estimation::{estimate as run_estimate, EstimationInput};
use fp_parser::scan_html;
use serde::Deserialize;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

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

pub async fn estimate(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let body: EstimateBody = req.json().await?;
    let result = run_estimate(body.signals);

    Response::from_json(&json!({ "result": result }))
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
