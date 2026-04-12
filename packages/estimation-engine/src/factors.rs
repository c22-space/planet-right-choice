/// Material emission factors in kg CO2e / kg of material.
/// Sources: ecoinvent 3.9, Higg MSI 2023, iNEMI, BNEF 2024.
pub struct MaterialFactor {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub kg_co2e_per_kg: f64,
}

pub static MATERIAL_FACTORS: &[MaterialFactor] = &[
    MaterialFactor {
        name: "Aluminium (virgin)",
        keywords: &["aluminium", "aluminum"],
        kg_co2e_per_kg: 11.5,
    },
    MaterialFactor {
        name: "Aluminium (recycled)",
        keywords: &["recycled aluminium", "recycled aluminum"],
        kg_co2e_per_kg: 0.9,
    },
    MaterialFactor {
        name: "Steel",
        keywords: &["steel", "stainless steel", "iron"],
        kg_co2e_per_kg: 2.3,
    },
    MaterialFactor {
        name: "ABS plastic",
        keywords: &["abs", "abs plastic", "plastic"],
        kg_co2e_per_kg: 3.7,
    },
    MaterialFactor {
        name: "Recycled PET",
        keywords: &["recycled pet", "rpet", "recycled polyester"],
        kg_co2e_per_kg: 1.3,
    },
    MaterialFactor {
        name: "Virgin PET / Polyester",
        keywords: &["pet", "polyester", "nylon"],
        kg_co2e_per_kg: 5.5,
    },
    MaterialFactor {
        name: "Cotton (conventional)",
        keywords: &["cotton"],
        kg_co2e_per_kg: 8.0,
    },
    MaterialFactor {
        name: "Organic cotton",
        keywords: &["organic cotton"],
        kg_co2e_per_kg: 5.5,
    },
    MaterialFactor {
        name: "Wool",
        keywords: &["wool", "merino"],
        kg_co2e_per_kg: 27.0,
    },
    MaterialFactor {
        name: "Glass",
        keywords: &["glass"],
        kg_co2e_per_kg: 0.85,
    },
    MaterialFactor {
        name: "Paper / Cardboard",
        keywords: &["paper", "cardboard", "kraft"],
        kg_co2e_per_kg: 1.1,
    },
    MaterialFactor {
        name: "PCB / Electronics",
        keywords: &["pcb", "circuit board", "electronics"],
        kg_co2e_per_kg: 35.0,
    },
];

/// Manufacturing overhead multipliers by broad product category.
pub fn manufacturing_multiplier(category_slug: &str) -> f64 {
    if category_slug.starts_with("electronics") {
        1.4
    } else if category_slug.starts_with("clothing") || category_slug.starts_with("apparel") {
        1.2
    } else {
        1.3
    }
}

/// Transport emission factor by known origin country (kg CO2e per kg per 1000 km, sea freight).
pub fn transport_ef_per_tonne_km(origin_country: Option<&str>) -> f64 {
    // Sea freight baseline: ~0.010 kg CO2e / tonne-km
    // Air freight: ~0.602 kg CO2e / tonne-km
    // We default to sea freight as most consumer goods travel by sea.
    let _ = origin_country; // Reserved for future differentiation
    0.000010 // kg CO2e / kg / km
}

/// Approximate shipping distance in km from major manufacturing hubs to Europe/US.
pub fn origin_distance_km(country: &str) -> f64 {
    match country.to_uppercase().as_str() {
        "CN" => 20_000.0, // China → Europe avg
        "BD" => 16_000.0, // Bangladesh
        "IN" => 12_000.0, // India
        "VN" => 18_000.0, // Vietnam
        "TW" => 18_000.0, // Taiwan
        "KR" => 18_000.0, // South Korea
        "JP" => 18_000.0, // Japan
        "DE" | "FR" | "IT" | "ES" | "NL" => 1_000.0, // European — regional
        "GB" => 1_000.0,
        "US" | "CA" => 5_000.0, // North America → Europe
        _ => 15_000.0, // unknown — conservative global avg
    }
}

/// Category intensity factors (kg CO2e / kg of product) for Tier 2 estimation.
/// Source: aggregated ecoinvent 3.9, GHG Protocol Scope 3 Cat 1 averages.
pub struct CategoryFactor {
    pub slug: &'static str,
    pub intensity_kg_per_kg: f64,
    pub scope: &'static str,
}

pub static CATEGORY_FACTORS: &[CategoryFactor] = &[
    CategoryFactor { slug: "electronics/smartphones", intensity_kg_per_kg: 42.0, scope: "lifecycle" },
    CategoryFactor { slug: "electronics/laptops", intensity_kg_per_kg: 28.0, scope: "lifecycle" },
    CategoryFactor { slug: "electronics/tablets", intensity_kg_per_kg: 30.0, scope: "lifecycle" },
    CategoryFactor { slug: "electronics/headphones", intensity_kg_per_kg: 22.0, scope: "lifecycle" },
    CategoryFactor { slug: "electronics/cameras", intensity_kg_per_kg: 25.0, scope: "lifecycle" },
    CategoryFactor { slug: "electronics/general", intensity_kg_per_kg: 30.0, scope: "lifecycle" },
    CategoryFactor { slug: "clothing/tshirts", intensity_kg_per_kg: 9.5, scope: "lifecycle" },
    CategoryFactor { slug: "clothing/jeans", intensity_kg_per_kg: 23.0, scope: "lifecycle" },
    CategoryFactor { slug: "clothing/shoes", intensity_kg_per_kg: 14.0, scope: "lifecycle" },
    CategoryFactor { slug: "clothing/jackets", intensity_kg_per_kg: 18.0, scope: "lifecycle" },
    CategoryFactor { slug: "clothing/general", intensity_kg_per_kg: 12.0, scope: "lifecycle" },
    CategoryFactor { slug: "appliances/washing-machine", intensity_kg_per_kg: 6.5, scope: "lifecycle" },
    CategoryFactor { slug: "appliances/refrigerator", intensity_kg_per_kg: 5.8, scope: "lifecycle" },
    CategoryFactor { slug: "appliances/general", intensity_kg_per_kg: 6.0, scope: "lifecycle" },
    CategoryFactor { slug: "furniture/chair", intensity_kg_per_kg: 4.2, scope: "lifecycle" },
    CategoryFactor { slug: "furniture/desk", intensity_kg_per_kg: 3.8, scope: "lifecycle" },
    CategoryFactor { slug: "furniture/general", intensity_kg_per_kg: 4.0, scope: "lifecycle" },
    CategoryFactor { slug: "food/general", intensity_kg_per_kg: 3.5, scope: "cradle-to-gate" },
    CategoryFactor { slug: "general", intensity_kg_per_kg: 10.0, scope: "lifecycle" },
];

/// USEEIO v2.1 spend-based intensity factors (kg CO2e / USD) for Tier 3.
/// Source: US EPA USEEIO v2.1, NAICS-mapped category averages.
pub struct SpendFactor {
    pub keywords: &'static [&'static str],
    pub kg_co2e_per_usd: f64,
}

pub static SPEND_FACTORS: &[SpendFactor] = &[
    SpendFactor { keywords: &["phone", "smartphone", "mobile"], kg_co2e_per_usd: 0.15 },
    SpendFactor { keywords: &["laptop", "computer", "tablet"], kg_co2e_per_usd: 0.12 },
    SpendFactor { keywords: &["electronics", "electronic"], kg_co2e_per_usd: 0.14 },
    SpendFactor { keywords: &["clothing", "shirt", "jeans", "dress", "shoes", "jacket"], kg_co2e_per_usd: 0.20 },
    SpendFactor { keywords: &["furniture", "chair", "table", "sofa"], kg_co2e_per_usd: 0.18 },
    SpendFactor { keywords: &["appliance", "washer", "fridge", "dishwasher"], kg_co2e_per_usd: 0.16 },
    SpendFactor { keywords: &["food", "grocery"], kg_co2e_per_usd: 0.45 },
    SpendFactor { keywords: &["toy", "game"], kg_co2e_per_usd: 0.22 },
];
