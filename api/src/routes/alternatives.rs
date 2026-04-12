use crate::middleware::auth::api_key_required;
use crate::db::models::Product;
use crate::services::scoring::{find_alternatives, rank_products};
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    let category = params.get("category").cloned().unwrap_or_default();
    let baseline: f64 = params
        .get("baselineCo2eKg")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::INFINITY);
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
        .min(10);

    let db = ctx.env.d1("DB")?;

    let (cat_avg, category_id) = if !category.is_empty() {
        let row = db
            .prepare("SELECT id, avg_co2e_kg FROM categories WHERE slug = ?1")
            .bind(&[category.clone().into()])?
            .first::<serde_json::Value>(None)
            .await?;

        let avg = row.as_ref().and_then(|r| r["avg_co2e_kg"].as_f64());
        let id = row.and_then(|r| r["id"].as_i64());
        (avg, id)
    } else {
        (None, None)
    };

    let mut query = "SELECT p.* FROM products p
                     JOIN categories c ON p.category_id = c.id
                     WHERE p.is_active = 1 AND p.co2e_kg IS NOT NULL".to_string();

    let mut bindings: Vec<worker::wasm_bindgen::JsValue> = Vec::new();

    if let Some(id) = category_id {
        query.push_str(" AND p.category_id = ?1 ORDER BY p.co2e_kg ASC LIMIT 50");
        bindings.push(id.into());
    } else {
        query.push_str(" ORDER BY p.co2e_kg ASC LIMIT 50");
    }

    let stmt = db.prepare(&query);
    let results = if bindings.is_empty() {
        stmt.all().await?
    } else {
        stmt.bind(&bindings)?.all().await?
    };

    let products: Vec<Product> = results.results()?;
    let ranked = rank_products(products, cat_avg);
    let alternatives = find_alternatives(baseline, ranked, limit);

    Response::from_json(&json!({ "alternatives": alternatives }))
}

pub async fn by_asin(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = api_key_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let asin = ctx.param("asin").map_or("", |v| v).to_string();
    let db = ctx.env.d1("DB")?;

    let product = db
        .prepare(
            "SELECT p.*, c.avg_co2e_kg as cat_avg FROM products p
             JOIN categories c ON p.category_id = c.id
             WHERE p.asin = ?1 AND p.is_active = 1",
        )
        .bind(&[asin.clone().into()])?
        .first::<serde_json::Value>(None)
        .await?;

    let Some(product_val) = product else {
        return Response::from_json(&json!({ "alternatives": [], "baseline": null }));
    };

    let co2e_kg = product_val["co2e_kg"].as_f64().unwrap_or(0.0);
    let cat_avg = product_val["cat_avg"].as_f64();
    let cat_id = product_val["category_id"].as_i64().unwrap_or(0);

    let peers: Vec<Product> = db
        .prepare(
            "SELECT p.* FROM products p
             WHERE p.category_id = ?1 AND p.is_active = 1 AND p.co2e_kg IS NOT NULL AND p.asin != ?2
             ORDER BY p.co2e_kg ASC LIMIT 30",
        )
        .bind(&[cat_id.into(), asin.clone().into()])?
        .all()
        .await?
        .results()?;

    let ranked = rank_products(peers, cat_avg);
    let alternatives = find_alternatives(co2e_kg, ranked, 3);

    Response::from_json(&json!({
        "alternatives": alternatives,
        "baseline": { "asin": asin, "co2eKg": co2e_kg }
    }))
}
