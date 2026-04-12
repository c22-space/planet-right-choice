use crate::db::models::Product;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredProduct {
    pub product: Product,
    pub co2e_kg: f64,
    pub vs_category: f64,
    pub percentile: u32,
    pub saving_kg: Option<f64>,
}

/// Rank products by CO2e and assign percentiles. Lower CO2e = greener = higher percentile.
pub fn rank_products(products: Vec<Product>, category_avg_co2e: Option<f64>) -> Vec<ScoredProduct> {
    let mut scored: Vec<(Product, f64)> = products
        .into_iter()
        .filter_map(|p| p.co2e_kg.map(|kg| (p, kg)))
        .collect();

    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let n = scored.len();
    scored
        .into_iter()
        .enumerate()
        .map(|(i, (product, co2e_kg))| {
            let vs_category = category_avg_co2e
                .map(|avg| if avg > 0.0 { co2e_kg / avg } else { 1.0 })
                .unwrap_or(1.0);
            let percentile = if n <= 1 {
                100
            } else {
                ((n - 1 - i) * 100 / (n - 1)) as u32
            };
            ScoredProduct {
                product,
                co2e_kg,
                vs_category,
                percentile,
                saving_kg: None,
            }
        })
        .collect()
}

/// Find greener alternatives relative to a baseline CO2e value.
pub fn find_alternatives(
    baseline_co2e_kg: f64,
    ranked: Vec<ScoredProduct>,
    limit: usize,
) -> Vec<ScoredProduct> {
    ranked
        .into_iter()
        .filter(|s| s.co2e_kg < baseline_co2e_kg)
        .take(limit)
        .map(|mut s| {
            s.saving_kg = Some(baseline_co2e_kg - s.co2e_kg);
            s
        })
        .collect()
}
