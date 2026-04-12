use crate::classifier::classify;
use crate::factors::{
    manufacturing_multiplier, origin_distance_km, transport_ef_per_tonne_km,
    CATEGORY_FACTORS, MATERIAL_FACTORS, SPEND_FACTORS,
};
use crate::signals::PageSignals;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstimationTier {
    /// Material-based hybrid method — highest confidence
    One = 1,
    /// Weight-scaled category average — medium confidence
    Two = 2,
    /// Spend-based EEIO — lowest confidence
    Three = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimationResult {
    pub estimated_co2e_kg: f64,
    pub confidence: f64,
    pub tier: EstimationTier,
    pub method_version: String,
    pub category_slug: String,
}

/// Run the tiered estimation protocol on page signals.
/// Tries Tier 1 → 2 → 3 in order of confidence.
pub fn run_estimation(signals: &PageSignals) -> EstimationResult {
    let category_slug = classify(signals);

    // Tier 1: material-based (requires weight + material hints)
    if let Some(result) = try_tier1(signals, &category_slug) {
        return result;
    }

    // Tier 2: weight-scaled category average (requires weight)
    if let Some(result) = try_tier2(signals, &category_slug) {
        return result;
    }

    // Tier 3: spend-based EEIO fallback
    tier3(signals, &category_slug)
}

fn try_tier1(signals: &PageSignals, category_slug: &str) -> Option<EstimationResult> {
    let weight_kg = signals.weight_kg?;
    if signals.material_hints.is_empty() {
        return None;
    }

    let mut material_co2e = 0.0f64;
    let mut matched_any = false;

    for hint in &signals.material_hints {
        let hint_lower = hint.to_lowercase();
        // Try longest-match first (most specific)
        let mut best: Option<f64> = None;
        let mut best_len = 0usize;

        for factor in MATERIAL_FACTORS {
            for &kw in factor.keywords {
                if hint_lower.contains(kw) && kw.len() > best_len {
                    best = Some(factor.kg_co2e_per_kg);
                    best_len = kw.len();
                }
            }
        }

        if let Some(ef) = best {
            // We don't have exact percentages, so distribute weight evenly
            let share = weight_kg / signals.material_hints.len() as f64;
            material_co2e += share * ef;
            matched_any = true;
        }
    }

    if !matched_any {
        return None;
    }

    let multiplier = manufacturing_multiplier(category_slug);
    let transport_ef = transport_ef_per_tonne_km(signals.origin_country.as_deref());
    let distance = signals
        .origin_country
        .as_deref()
        .map(origin_distance_km)
        .unwrap_or(15_000.0);
    let transport_co2e = weight_kg * transport_ef * distance;

    let total = material_co2e * multiplier + transport_co2e;
    let confidence = compute_confidence(signals, EstimationTier::One);

    Some(EstimationResult {
        estimated_co2e_kg: round2(total),
        confidence,
        tier: EstimationTier::One,
        method_version: PROTOCOL_VERSION.into(),
        category_slug: category_slug.into(),
    })
}

fn try_tier2(signals: &PageSignals, category_slug: &str) -> Option<EstimationResult> {
    let weight_kg = signals.weight_kg?;

    let intensity = CATEGORY_FACTORS
        .iter()
        .find(|f| f.slug == category_slug)
        .map(|f| f.intensity_kg_per_kg)
        .unwrap_or_else(|| {
            CATEGORY_FACTORS
                .iter()
                .find(|f| f.slug == "general")
                .map(|f| f.intensity_kg_per_kg)
                .unwrap_or(10.0)
        });

    let total = weight_kg * intensity;
    let confidence = compute_confidence(signals, EstimationTier::Two);

    Some(EstimationResult {
        estimated_co2e_kg: round2(total),
        confidence,
        tier: EstimationTier::Two,
        method_version: PROTOCOL_VERSION.into(),
        category_slug: category_slug.into(),
    })
}

fn tier3(signals: &PageSignals, category_slug: &str) -> EstimationResult {
    let price = signals.price_usd.unwrap_or(50.0); // $50 fallback

    // Find best matching spend factor
    let category_text = format!(
        "{} {} {}",
        signals.product_name.as_deref().unwrap_or(""),
        category_slug,
        signals
            .category_breadcrumb
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();

    let intensity = SPEND_FACTORS
        .iter()
        .find(|f| f.keywords.iter().any(|kw| category_text.contains(kw)))
        .map(|f| f.kg_co2e_per_usd)
        .unwrap_or(0.18); // broad consumer goods default

    let total = price * intensity;
    let confidence = compute_confidence(signals, EstimationTier::Three);

    EstimationResult {
        estimated_co2e_kg: round2(total),
        confidence,
        tier: EstimationTier::Three,
        method_version: PROTOCOL_VERSION.into(),
        category_slug: category_slug.into(),
    }
}

fn compute_confidence(signals: &PageSignals, tier: EstimationTier) -> f64 {
    let mut base = match tier {
        EstimationTier::One => 0.75,
        EstimationTier::Two => 0.52,
        EstimationTier::Three => 0.27,
    };

    if signals.origin_country.is_some() {
        base += 0.04;
    }
    if signals.weight_kg.is_some() && tier != EstimationTier::Three {
        base += 0.03;
    }
    if !signals.material_hints.is_empty() {
        base += 0.03;
    }

    // Never claim certainty without fp: tags — cap at 0.88
    f64::min(base, 0.88)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals_with_weight_and_material() -> PageSignals {
        PageSignals {
            product_name: Some("Organic Cotton T-Shirt".into()),
            weight_kg: Some(0.3),
            material_hints: vec!["organic cotton".into()],
            category_breadcrumb: vec!["Clothing".into(), "T-Shirts".into()],
            domain: "example.com".into(),
            page_url_hash: "abc123".into(),
            session_id: "sess1".into(),
            ..Default::default()
        }
    }

    #[test]
    fn tier1_runs_for_material_signals() {
        let signals = signals_with_weight_and_material();
        let result = run_estimation(&signals);
        assert_eq!(result.tier, EstimationTier::One);
        assert!(result.estimated_co2e_kg > 0.0);
        assert!(result.confidence >= 0.75);
    }

    #[test]
    fn tier2_runs_without_materials() {
        let signals = PageSignals {
            product_name: Some("Blue Jeans".into()),
            weight_kg: Some(0.8),
            category_breadcrumb: vec!["Clothing".into(), "Jeans".into()],
            domain: "example.com".into(),
            page_url_hash: "abc123".into(),
            session_id: "sess1".into(),
            ..Default::default()
        };
        let result = run_estimation(&signals);
        assert_eq!(result.tier, EstimationTier::Two);
    }

    #[test]
    fn tier3_runs_with_only_price() {
        let signals = PageSignals {
            product_name: Some("Something".into()),
            price_usd: Some(99.0),
            domain: "example.com".into(),
            page_url_hash: "abc123".into(),
            session_id: "sess1".into(),
            ..Default::default()
        };
        let result = run_estimation(&signals);
        assert_eq!(result.tier, EstimationTier::Three);
        assert!(result.estimated_co2e_kg > 0.0);
    }

    #[test]
    fn confidence_capped_at_088() {
        let signals = signals_with_weight_and_material();
        let result = run_estimation(&signals);
        assert!(result.confidence <= 0.88);
    }
}
