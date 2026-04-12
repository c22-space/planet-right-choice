use crate::db::models::Product;
use crate::middleware::auth::admin_required;
use serde::Deserialize;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

#[derive(Deserialize)]
struct ProductBody {
    name: String,
    brand: String,
    category_id: i64,
    asin: Option<String>,
    url: Option<String>,
    image_url: Option<String>,
    description: Option<String>,
    co2e_kg: Option<f64>,
    co2e_scope: Option<String>,
    co2e_source: Option<String>,
    co2e_confidence: Option<f64>,
    certifications: Option<Vec<String>>,
    materials: Option<Vec<serde_json::Value>>,
    weight_kg: Option<f64>,
    origin_country: Option<String>,
}

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let page: u32 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("pageSize").and_then(|v| v.parse().ok()).unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;
    let include_inactive = params.get("includeInactive").map(|v| v == "true").unwrap_or(false);

    let db = ctx.env.d1("DB")?;
    let where_clause = if include_inactive { "" } else { "WHERE p.is_active = 1" };

    let total = db
        .prepare(&format!("SELECT COUNT(*) as n FROM products p {where_clause}"))
        .first::<serde_json::Value>(None)
        .await?
        .and_then(|v| v["n"].as_i64())
        .unwrap_or(0);

    let products: Vec<Product> = db
        .prepare(&format!(
            "SELECT p.*, c.slug as category_slug FROM products p
             JOIN categories c ON p.category_id = c.id
             {where_clause}
             ORDER BY p.updated_at DESC LIMIT ?1 OFFSET ?2"
        ))
        .bind(&[(page_size as f64).into(), (offset as f64).into()])?
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({ "data": products, "page": page, "pageSize": page_size, "total": total }))
}

pub async fn create(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let body: ProductBody = req.json().await?;
    let db = ctx.env.d1("DB")?;

    let certs = serde_json::to_string(&body.certifications.unwrap_or_default()).unwrap_or_default();
    let mats = serde_json::to_string(&body.materials.unwrap_or_default()).unwrap_or_default();

    let result = db.prepare(
        "INSERT INTO products (name, brand, category_id, asin, url, image_url, description,
                               co2e_kg, co2e_scope, co2e_source, co2e_confidence,
                               certifications, materials, weight_kg, origin_country)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         RETURNING id"
    ).bind(&[
        body.name.into(), body.brand.into(), body.category_id.into(),
        body.asin.unwrap_or_default().into(), body.url.unwrap_or_default().into(),
        body.image_url.unwrap_or_default().into(), body.description.unwrap_or_default().into(),
        body.co2e_kg.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.co2e_scope.unwrap_or_default().into(), body.co2e_source.unwrap_or_default().into(),
        body.co2e_confidence.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        certs.into(), mats.into(),
        body.weight_kg.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.origin_country.unwrap_or_default().into(),
    ])?.first::<serde_json::Value>(None).await?;

    let id = result.and_then(|v| v["id"].as_i64()).unwrap_or(0);
    Response::from_json(&json!({ "id": id }))
}

pub async fn update(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let id: i64 = ctx.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);
    let body: ProductBody = req.json().await?;
    let db = ctx.env.d1("DB")?;

    let certs = serde_json::to_string(&body.certifications.unwrap_or_default()).unwrap_or_default();
    let mats = serde_json::to_string(&body.materials.unwrap_or_default()).unwrap_or_default();

    db.prepare(
        "UPDATE products SET name=?1, brand=?2, category_id=?3, asin=?4, url=?5,
                             image_url=?6, description=?7, co2e_kg=?8, co2e_scope=?9,
                             co2e_source=?10, co2e_confidence=?11, certifications=?12,
                             materials=?13, weight_kg=?14, origin_country=?15,
                             updated_at=datetime('now')
         WHERE id=?16"
    ).bind(&[
        body.name.into(), body.brand.into(), body.category_id.into(),
        body.asin.unwrap_or_default().into(), body.url.unwrap_or_default().into(),
        body.image_url.unwrap_or_default().into(), body.description.unwrap_or_default().into(),
        body.co2e_kg.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.co2e_scope.unwrap_or_default().into(), body.co2e_source.unwrap_or_default().into(),
        body.co2e_confidence.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        certs.into(), mats.into(),
        body.weight_kg.map(|v| worker::wasm_bindgen::JsValue::from_f64(v)).unwrap_or(worker::wasm_bindgen::JsValue::NULL),
        body.origin_country.unwrap_or_default().into(),
        id.into(),
    ])?.run().await?;

    Response::from_json(&json!({ "ok": true }))
}

pub async fn delete(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let id: i64 = ctx.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);
    let db = ctx.env.d1("DB")?;

    db.prepare("UPDATE products SET is_active = 0, updated_at = datetime('now') WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    Response::from_json(&json!({ "ok": true }))
}
