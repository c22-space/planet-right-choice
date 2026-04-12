use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Co2eScope {
    Lifecycle,
    CradleToGate,
    UsePhase,
    EndOfLife,
}

impl Default for Co2eScope {
    fn default() -> Self {
        Self::Lifecycle
    }
}

impl Co2eScope {
    pub fn from_str(s: &str) -> Self {
        match s {
            "lifecycle" => Self::Lifecycle,
            "cradle-to-gate" => Self::CradleToGate,
            "use-phase" => Self::UsePhase,
            "end-of-life" => Self::EndOfLife,
            _ => Self::Lifecycle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpProduct {
    pub product: String,
    pub co2e_kg: f64,
    pub scope: Co2eScope,
    pub certifier: Option<String>,
    pub fp_version: Option<String>,
    pub raw_tags: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing required tag: {0}")]
    MissingTag(&'static str),
    #[error("invalid co2e value: {0}")]
    InvalidCo2e(String),
}

/// Parse a map of fp:* tag values into a `FpProduct`.
/// Returns `None` (not an error) if the minimum required tags aren't present.
pub fn parse_fp_tags(tags: &HashMap<String, String>) -> Result<FpProduct, ParseError> {
    let product = tags
        .get("fp:product")
        .ok_or(ParseError::MissingTag("fp:product"))?
        .clone();

    let co2e_raw = tags
        .get("fp:co2e")
        .ok_or(ParseError::MissingTag("fp:co2e"))?;

    let co2e_value: f64 = co2e_raw
        .parse()
        .map_err(|_| ParseError::InvalidCo2e(co2e_raw.clone()))?;

    if co2e_value < 0.0 {
        return Err(ParseError::InvalidCo2e(format!("negative value: {co2e_value}")));
    }

    let unit = tags.get("fp:co2e:unit").map(|s| s.as_str()).unwrap_or("kg");
    let co2e_kg = match unit.to_lowercase().as_str() {
        "g" => co2e_value / 1000.0,
        "t" => co2e_value * 1000.0,
        _ => co2e_value, // assume kg
    };

    let scope = tags
        .get("fp:scope")
        .map(|s| Co2eScope::from_str(s))
        .unwrap_or_default();

    Ok(FpProduct {
        product,
        co2e_kg,
        scope,
        certifier: tags.get("fp:certifier").cloned(),
        fp_version: tags.get("fp:version").cloned(),
        raw_tags: tags.clone(),
    })
}

/// Extract fp:* meta tags from raw HTML using a lightweight regex-free scanner.
/// Returns an empty map if no tags are found.
pub fn extract_tags_from_html(html: &str) -> HashMap<String, String> {
    let mut tags = HashMap::new();

    // We scan for <meta ... property="fp:*" ... content="..." ...>
    // Handles both attribute orderings and single/double quotes.
    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Find next '<meta'
        let Some(meta_start) = find_substr(&bytes[i..], b"<meta") else {
            break;
        };
        let tag_start = i + meta_start;

        // Find end of this tag
        let Some(tag_len) = find_substr(&bytes[tag_start..], b">") else {
            break;
        };
        let tag_end = tag_start + tag_len + 1;
        i = tag_end;

        let tag_slice = &html[tag_start..tag_end];

        // Extract property and content attributes
        let property = extract_attr(tag_slice, "property");
        let content = extract_attr(tag_slice, "content");

        if let (Some(prop), Some(cont)) = (property, content) {
            if prop.starts_with("fp:") {
                tags.insert(prop, cont);
            }
        }
    }

    tags
}

fn find_substr(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w| w.eq_ignore_ascii_case(needle))
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    // Match attr="value" or attr='value'
    let lower = tag.to_lowercase();
    let pattern = format!("{attr}=");
    let pos = lower.find(&pattern)?;
    let after = &tag[pos + pattern.len()..];

    let (quote, rest) = if after.starts_with('"') {
        ('"', &after[1..])
    } else if after.starts_with('\'') {
        ('\'', &after[1..])
    } else {
        // unquoted — read to next space or >
        let end = after.find(|c: char| c == ' ' || c == '>').unwrap_or(after.len());
        return Some(after[..end].to_string());
    };

    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// Convenience: parse fp: tags directly from raw HTML.
pub fn scan_html(html: &str) -> Option<FpProduct> {
    let tags = extract_tags_from_html(html);
    if tags.is_empty() {
        return None;
    }
    parse_fp_tags(&tags).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_fp_tags() {
        let mut tags = HashMap::new();
        tags.insert("fp:product".into(), "Fairphone 5".into());
        tags.insert("fp:co2e".into(), "23.6".into());
        tags.insert("fp:co2e:unit".into(), "kg".into());
        tags.insert("fp:scope".into(), "lifecycle".into());

        let result = parse_fp_tags(&tags).unwrap();
        assert_eq!(result.product, "Fairphone 5");
        assert!((result.co2e_kg - 23.6).abs() < 0.001);
        assert_eq!(result.scope, Co2eScope::Lifecycle);
    }

    #[test]
    fn converts_grams_to_kg() {
        let mut tags = HashMap::new();
        tags.insert("fp:product".into(), "Test".into());
        tags.insert("fp:co2e".into(), "500".into());
        tags.insert("fp:co2e:unit".into(), "g".into());

        let result = parse_fp_tags(&tags).unwrap();
        assert!((result.co2e_kg - 0.5).abs() < 0.001);
    }

    #[test]
    fn returns_error_for_missing_product() {
        let mut tags = HashMap::new();
        tags.insert("fp:co2e".into(), "10".into());
        assert!(parse_fp_tags(&tags).is_err());
    }

    #[test]
    fn scans_html_for_tags() {
        let html = r#"<html><head>
            <meta property="fp:product" content="Fairphone 5" />
            <meta property="fp:co2e" content="23.6" />
            <meta property="fp:scope" content="lifecycle" />
        </head></html>"#;

        let product = scan_html(html).unwrap();
        assert_eq!(product.product, "Fairphone 5");
    }
}
