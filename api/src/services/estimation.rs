use estimation_engine::{run_estimation, PageSignals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstimationInput {
    pub product_name: Option<String>,
    pub brand: Option<String>,
    pub category_breadcrumb: Vec<String>,
    pub amazon_category: Option<String>,
    pub weight_kg: Option<f64>,
    pub material_hints: Vec<String>,
    pub origin_country: Option<String>,
    pub price_usd: Option<f64>,
    pub asin: Option<String>,
    pub domain: String,
    pub page_url_hash: String,
    pub session_id: String,
}

impl From<EstimationInput> for PageSignals {
    fn from(i: EstimationInput) -> Self {
        Self {
            product_name: i.product_name,
            brand: i.brand,
            category_breadcrumb: i.category_breadcrumb,
            amazon_category: i.amazon_category,
            weight_kg: i.weight_kg,
            material_hints: i.material_hints,
            origin_country: i.origin_country,
            price_usd: i.price_usd,
            asin: i.asin,
            domain: i.domain,
            page_url_hash: i.page_url_hash,
            session_id: i.session_id,
        }
    }
}

pub fn estimate(input: EstimationInput) -> estimation_engine::EstimationResult {
    let signals: PageSignals = input.into();
    run_estimation(&signals)
}
