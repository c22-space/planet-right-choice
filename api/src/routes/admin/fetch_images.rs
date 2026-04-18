use crate::middleware::auth::admin_required;
use crate::services::paapi::get_item_images;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

/// Admin: fetch product images from Amazon PA API and store in DB.
/// POST /v1/admin/catalogue/sync-images
pub async fn sync_images(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(r) = admin_required(&req, &ctx.env)? {
        return Ok(r);
    }

    let access_key = ctx.env.secret("PAAPI_ACCESS_KEY")?.to_string();
    let secret_key = ctx.env.secret("PAAPI_SECRET_KEY")?.to_string();
    let partner_tag = ctx.env.secret("PAAPI_PARTNER_TAG")?.to_string();

    let db = ctx.env.d1("DB")?;

    // Fetch all products with an ASIN
    let rows: Vec<serde_json::Value> = db
        .prepare("SELECT id, asin FROM products WHERE asin IS NOT NULL AND is_active = 1")
        .all()
        .await?
        .results()?;

    let products: Vec<(i64, String)> = rows
        .into_iter()
        .filter_map(|r| {
            let id = r["id"].as_i64()?;
            let asin = r["asin"].as_str()?.to_string();
            Some((id, asin))
        })
        .collect();

    let mut updated = 0usize;
    let mut errors = 0usize;

    // PA API accepts up to 10 ASINs per request
    for chunk in products.chunks(10) {
        let asins: Vec<String> = chunk.iter().map(|(_, a)| a.clone()).collect();
        match get_item_images(&access_key, &secret_key, &partner_tag, &asins).await {
            Ok(images) => {
                for (id, asin) in chunk {
                    if let Some(img_url) = images.get(asin) {
                        let _ = db
                            .prepare(
                                "UPDATE products SET image_url = ?1, updated_at = datetime('now') WHERE id = ?2",
                            )
                            .bind(&[img_url.clone().into(), (*id as f64).into()])?
                            .run()
                            .await;
                        updated += 1;
                    }
                }
            }
            Err(_) => {
                errors += chunk.len();
            }
        }
    }

    Response::from_json(&json!({
        "ok": true,
        "updated": updated,
        "errors": errors,
        "total": products.len(),
    }))
}
