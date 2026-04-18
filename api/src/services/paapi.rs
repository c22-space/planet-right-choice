use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use worker::{Fetch, Headers, Method, Request, RequestInit};

type HmacSha256 = Hmac<Sha256>;

const HOST: &str = "webservices.amazon.in";
const REGION: &str = "eu-west-1";
const SERVICE: &str = "ProductAdvertisingAPI";
const TARGET: &str = "com.amazon.paapi5.v1.ProductAdvertisingAPIv1.GetItems";
const MARKETPLACE: &str = "www.amazon.in";

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn sha256_hex(data: &[u8]) -> String {
    hex_encode(&Sha256::digest(data))
}

fn hmac_sha256_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(secret_key: &str, date: &str) -> Vec<u8> {
    let k = hmac_sha256_bytes(format!("AWS4{secret_key}").as_bytes(), date.as_bytes());
    let k = hmac_sha256_bytes(&k, REGION.as_bytes());
    let k = hmac_sha256_bytes(&k, SERVICE.as_bytes());
    hmac_sha256_bytes(&k, b"aws4_request")
}

fn authorization(access_key: &str, secret_key: &str, amz_date: &str, body: &str) -> String {
    let date = &amz_date[..8]; // YYYYMMDD

    let body_hash = sha256_hex(body.as_bytes());

    let canonical = format!(
        "POST\n/paapi5/getitems\n\ncontent-type:application/json; charset=utf-8\nhost:{HOST}\nx-amz-date:{amz_date}\nx-amz-target:{TARGET}\n\ncontent-type;host;x-amz-date;x-amz-target\n{body_hash}"
    );

    let credential_scope = format!("{date}/{REGION}/{SERVICE}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical.as_bytes())
    );

    let sig = hex_encode(&hmac_sha256_bytes(&signing_key(secret_key, date), string_to_sign.as_bytes()));

    format!(
        "AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, SignedHeaders=content-type;host;x-amz-date;x-amz-target, Signature={sig}"
    )
}

fn amz_date_now() -> String {
    // Use JS Date via wasm_bindgen since we're in a Worker
    let ms = worker::js_sys::Date::now() as u64;
    let secs = ms / 1000;
    // Format as YYYYMMDDTHHmmssZ
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400; // days since epoch

    // Compute calendar date from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days);

    format!("{year:04}{month:02}{day:02}T{hour:02}{min:02}{sec:02}Z")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Gregorian calendar calculation
    let mut d = days;
    let mut y = 1970u64;
    loop {
        let leap = is_leap(y);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    (y, (m + 1) as u64, d + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Fetch images for up to 10 ASINs from PA API v5.
/// Returns a map of ASIN -> image URL.
pub async fn get_item_images(
    access_key: &str,
    secret_key: &str,
    partner_tag: &str,
    asins: &[String],
) -> worker::Result<HashMap<String, String>> {
    let body = serde_json::json!({
        "ItemIds": asins,
        "Resources": ["Images.Primary.Large"],
        "PartnerTag": partner_tag,
        "PartnerType": "Associates",
        "Marketplace": MARKETPLACE,
    })
    .to_string();

    let amz_date = amz_date_now();
    let auth = authorization(access_key, secret_key, &amz_date, &body);

    let url = format!("https://{HOST}/paapi5/getitems");

    let headers = Headers::new();
    headers.set("Content-Type", "application/json; charset=utf-8")?;
    headers.set("Host", HOST)?;
    headers.set("X-Amz-Date", &amz_date)?;
    headers.set("X-Amz-Target", TARGET)?;
    headers.set("Authorization", &auth)?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_headers(headers);
    init.with_body(Some(worker::wasm_bindgen::JsValue::from_str(&body)));

    let req = Request::new_with_init(&url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;
    let json: serde_json::Value = resp.json().await?;

    let mut result = HashMap::new();
    if let Some(items) = json["ItemsResult"]["Items"].as_array() {
        for item in items {
            let asin = item["ASIN"].as_str().unwrap_or("").to_string();
            let img = item["Images"]["Primary"]["Large"]["URL"].as_str().unwrap_or("");
            if !asin.is_empty() && !img.is_empty() {
                result.insert(asin, img.to_string());
            }
        }
    }
    Ok(result)
}
