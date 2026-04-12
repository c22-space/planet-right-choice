use crate::signals::PageSignals;

/// Map page signals to the closest category slug from our taxonomy.
pub fn classify(signals: &PageSignals) -> String {
    let text = build_search_text(signals);

    // Check electronics first (most specific)
    if contains_any(&text, &["smartphone", "iphone", "android", "mobile phone", "cell phone"]) {
        return "electronics/smartphones".into();
    }
    if contains_any(&text, &["laptop", "macbook", "notebook", "chromebook"]) {
        return "electronics/laptops".into();
    }
    if contains_any(&text, &["tablet", "ipad"]) {
        return "electronics/tablets".into();
    }
    if contains_any(&text, &["headphone", "earphone", "earbud", "airpod"]) {
        return "electronics/headphones".into();
    }
    if contains_any(&text, &["camera", "dslr", "mirrorless"]) {
        return "electronics/cameras".into();
    }

    // Clothing
    if contains_any(&text, &["t-shirt", "tshirt", "tee shirt"]) {
        return "clothing/tshirts".into();
    }
    if contains_any(&text, &["jeans", "denim trousers", "denim pants"]) {
        return "clothing/jeans".into();
    }
    if contains_any(&text, &["shoes", "sneakers", "trainers", "boots", "footwear"]) {
        return "clothing/shoes".into();
    }
    if contains_any(&text, &["jacket", "coat", "hoodie", "puffer"]) {
        return "clothing/jackets".into();
    }

    // Appliances
    if contains_any(&text, &["washing machine", "washer", "laundry machine"]) {
        return "appliances/washing-machine".into();
    }
    if contains_any(&text, &["refrigerator", "fridge", "freezer"]) {
        return "appliances/refrigerator".into();
    }

    // Furniture
    if contains_any(&text, &["chair", "office chair", "armchair"]) {
        return "furniture/chair".into();
    }
    if contains_any(&text, &["desk", "writing desk", "standing desk"]) {
        return "furniture/desk".into();
    }

    // Broad fallbacks
    if contains_any(&text, &["electronics", "electronic", "gadget", "device"]) {
        return "electronics/general".into();
    }
    if contains_any(&text, &["clothing", "apparel", "fashion", "wear"]) {
        return "clothing/general".into();
    }
    if contains_any(&text, &["appliance", "kitchen"]) {
        return "appliances/general".into();
    }
    if contains_any(&text, &["furniture", "sofa", "shelf", "cabinet"]) {
        return "furniture/general".into();
    }

    "general".into()
}

fn build_search_text(signals: &PageSignals) -> String {
    let mut parts = Vec::new();

    if let Some(ref name) = signals.product_name {
        parts.push(name.to_lowercase());
    }
    if let Some(ref cat) = signals.amazon_category {
        parts.push(cat.to_lowercase());
    }
    for crumb in &signals.category_breadcrumb {
        parts.push(crumb.to_lowercase());
    }

    parts.join(" ")
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|t| text.contains(t))
}
