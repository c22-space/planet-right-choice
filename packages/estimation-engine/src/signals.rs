use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageSignals {
    pub product_name: Option<String>,
    pub brand: Option<String>,
    pub category_breadcrumb: Vec<String>,
    pub amazon_category: Option<String>,
    /// Weight in kilograms parsed from spec tables, product details, etc.
    pub weight_kg: Option<f64>,
    /// Strings like "recycled aluminium", "organic cotton", "ABS plastic"
    pub material_hints: Vec<String>,
    /// ISO 3166-1 alpha-2 country code if origin is detectable
    pub origin_country: Option<String>,
    pub price_usd: Option<f64>,
    pub asin: Option<String>,
    pub domain: String,
    /// SHA-256 of the page URL — never the raw URL
    pub page_url_hash: String,
    pub session_id: String,
}
