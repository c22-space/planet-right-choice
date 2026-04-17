use crate::db::models::{Category, Product};
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

pub async fn list_categories(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let categories: Vec<Category> = db
        .prepare(
            "SELECT id, slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source
             FROM categories ORDER BY slug",
        )
        .all()
        .await?
        .results()?;

    Response::from_json(&json!({ "categories": categories }))
}

pub async fn list_products(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    let page: u32 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let page_size: u32 = params
        .get("pageSize")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);
    let offset = (page - 1) * page_size;
    let category = params.get("category").cloned();
    let search = params.get("search").filter(|s| !s.is_empty()).map(|s| format!("%{s}%"));

    let db = ctx.env.d1("DB")?;

    let (products, total) = match (category, search) {
        (Some(cat), Some(q)) => {
            let total = db
                .prepare(
                    "SELECT COUNT(*) as n FROM products p
                     JOIN categories c ON p.category_id = c.id
                     WHERE p.is_active = 1 AND c.slug = ?1
                       AND (p.name LIKE ?2 OR p.brand LIKE ?2)",
                )
                .bind(&[cat.clone().into(), q.clone().into()])?
                .first::<serde_json::Value>(None).await?
                .and_then(|v| v["n"].as_i64()).unwrap_or(0);
            let rows: Vec<Product> = db
                .prepare(
                    "SELECT p.* FROM products p
                     JOIN categories c ON p.category_id = c.id
                     WHERE p.is_active = 1 AND c.slug = ?1
                       AND (p.name LIKE ?2 OR p.brand LIKE ?2)
                     ORDER BY p.co2e_kg ASC NULLS LAST LIMIT ?3 OFFSET ?4",
                )
                .bind(&[cat.into(), q.into(), (page_size as f64).into(), (offset as f64).into()])?
                .all().await?.results()?;
            (rows, total)
        }
        (Some(cat), None) => {
            let total = db
                .prepare(
                    "SELECT COUNT(*) as n FROM products p
                     JOIN categories c ON p.category_id = c.id
                     WHERE p.is_active = 1 AND c.slug = ?1",
                )
                .bind(&[cat.clone().into()])?
                .first::<serde_json::Value>(None).await?
                .and_then(|v| v["n"].as_i64()).unwrap_or(0);
            let rows: Vec<Product> = db
                .prepare(
                    "SELECT p.* FROM products p
                     JOIN categories c ON p.category_id = c.id
                     WHERE p.is_active = 1 AND c.slug = ?1
                     ORDER BY p.co2e_kg ASC NULLS LAST LIMIT ?2 OFFSET ?3",
                )
                .bind(&[cat.into(), (page_size as f64).into(), (offset as f64).into()])?
                .all().await?.results()?;
            (rows, total)
        }
        (None, Some(q)) => {
            let total = db
                .prepare(
                    "SELECT COUNT(*) as n FROM products
                     WHERE is_active = 1 AND (name LIKE ?1 OR brand LIKE ?1)",
                )
                .bind(&[q.clone().into()])?
                .first::<serde_json::Value>(None).await?
                .and_then(|v| v["n"].as_i64()).unwrap_or(0);
            let rows: Vec<Product> = db
                .prepare(
                    "SELECT * FROM products
                     WHERE is_active = 1 AND (name LIKE ?1 OR brand LIKE ?1)
                     ORDER BY co2e_kg ASC NULLS LAST LIMIT ?2 OFFSET ?3",
                )
                .bind(&[q.into(), (page_size as f64).into(), (offset as f64).into()])?
                .all().await?.results()?;
            (rows, total)
        }
        (None, None) => {
            let total = db
                .prepare("SELECT COUNT(*) as n FROM products WHERE is_active = 1")
                .first::<serde_json::Value>(None).await?
                .and_then(|v| v["n"].as_i64()).unwrap_or(0);
            let rows: Vec<Product> = db
                .prepare(
                    "SELECT * FROM products WHERE is_active = 1
                     ORDER BY co2e_kg ASC NULLS LAST LIMIT ?1 OFFSET ?2",
                )
                .bind(&[(page_size as f64).into(), (offset as f64).into()])?
                .all().await?.results()?;
            (rows, total)
        }
    };

    Response::from_json(&json!({
        "data": products,
        "page": page,
        "pageSize": page_size,
        "total": total,
    }))
}

pub async fn get_product(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id: i64 = ctx.param("id")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| worker::Error::RustError("invalid id".into()))?;

    let db = ctx.env.d1("DB")?;
    let mut rows: Vec<serde_json::Value> = db
        .prepare("SELECT * FROM products WHERE id = ?1 AND is_active = 1 LIMIT 1")
        .bind(&[(id as f64).into()])?
        .all()
        .await?
        .results()?;

    match rows.pop() {
        Some(p) => Response::from_json(&json!({ "product": p })),
        None => Response::error("Not found", 404),
    }
}
