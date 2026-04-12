use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub parent_id: Option<i64>,
    pub avg_co2e_kg: Option<f64>,
    pub avg_co2e_scope: Option<String>,
    pub factor_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub category_id: i64,
    pub asin: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub co2e_kg: Option<f64>,
    pub co2e_scope: Option<String>,
    pub co2e_source: Option<String>,
    pub co2e_confidence: Option<f64>,
    pub certifications: Option<String>, // JSON array string
    pub materials: Option<String>,      // JSON array string
    pub weight_kg: Option<f64>,
    pub origin_country: Option<String>,
    pub is_active: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliateRule {
    pub id: i64,
    pub source_asin: String,
    pub target_asin: Option<String>,
    pub affiliate_tag: String,
    pub reason: Option<String>,
    pub is_active: i64,
    pub priority: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpDetection {
    pub id: i64,
    pub session_id: String,
    pub domain: String,
    pub page_url_hash: String,
    pub product_name: Option<String>,
    pub co2e_kg: Option<f64>,
    pub co2e_scope: Option<String>,
    pub fp_version: Option<String>,
    pub raw_tags: Option<String>,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimation {
    pub id: i64,
    pub session_id: String,
    pub domain: String,
    pub category_id: Option<i64>,
    pub product_name: Option<String>,
    pub signals: String,
    pub estimated_co2e_kg: f64,
    pub confidence: f64,
    pub tier: i64,
    pub method_version: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub last_login_at: Option<String>,
}
