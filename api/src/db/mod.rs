pub mod models;

use worker::{D1Database, Result};
use models::Product;

#[allow(dead_code)]
pub async fn get_products_by_category(
    db: &D1Database,
    category_slug: &str,
    exclude_ids: &[i64],
    limit: u32,
) -> Result<Vec<Product>> {
    let base = "SELECT p.* FROM products p
                JOIN categories c ON p.category_id = c.id
                WHERE p.is_active = 1 AND p.co2e_kg IS NOT NULL
                AND c.slug = ?
                ORDER BY p.co2e_kg ASC LIMIT ?";

    let stmt = db.prepare(base).bind(&[
        category_slug.into(),
        (limit as f64).into(),
    ])?;

    let results = stmt.all().await?;
    let products: Vec<Product> = results.results()?;

    // Filter excluded IDs in Rust (simpler than dynamic SQL binding)
    Ok(products
        .into_iter()
        .filter(|p| !exclude_ids.contains(&p.id))
        .collect())
}
