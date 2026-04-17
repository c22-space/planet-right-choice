use worker::*;

mod db;
mod middleware;
mod routes;
mod services;

fn cors_headers() -> Headers {
    let mut h = Headers::new();
    h.set("Access-Control-Allow-Origin", "*").unwrap();
    h.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS").unwrap();
    h.set("Access-Control-Allow-Headers", "Content-Type, X-API-Key, Authorization").unwrap();
    h
}

async fn add_cors(resp: Result<Response>) -> Result<Response> {
    let mut r = resp?;
    let headers = r.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    Ok(r)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.method() == Method::Options {
        return Ok(Response::empty()?.with_headers(cors_headers()));
    }

    let router = Router::new();

    add_cors(router
        // Health
        .get_async("/v1/health", routes::health::handle)
        // Footprint
        .post_async("/v1/footprint/parse", routes::footprint::parse)
        .post_async("/v1/footprint/estimate", routes::footprint::estimate)
        .get_async("/v1/footprint/:product_id", routes::footprint::get_product_footprint)
        // Alternatives
        .get_async("/v1/alternatives", routes::alternatives::list)
        .get_async("/v1/alternatives/asin/:asin", routes::alternatives::by_asin)
        // Affiliate
        .get_async("/v1/affiliate/rules", routes::affiliate::get_rules)
        .post_async("/v1/affiliate/click", routes::affiliate::record_click)
        .get_async("/v1/affiliate/rewrite", routes::affiliate::rewrite)
        // Catalogue (public)
        .get_async("/v1/catalogue/categories", routes::catalogue::list_categories)
        .get_async("/v1/catalogue/products", routes::catalogue::list_products)
        .get_async("/v1/catalogue/products/:id", routes::catalogue::get_product)
        // Admin — auth handled inside each handler
        .post_async("/v1/admin/auth/login", routes::admin::auth::login)
        .get_async("/v1/admin/catalogue/products", routes::admin::catalogue::list)
        .post_async("/v1/admin/catalogue/products", routes::admin::catalogue::create)
        .put_async("/v1/admin/catalogue/products/:id", routes::admin::catalogue::update)
        .delete_async("/v1/admin/catalogue/products/:id", routes::admin::catalogue::delete)
        .get_async("/v1/admin/detections", routes::admin::detections::list)
        .get_async("/v1/admin/estimations", routes::admin::estimations::list)
        .get_async("/v1/admin/affiliate/rules", routes::admin::affiliate::list_rules)
        .post_async("/v1/admin/affiliate/rules", routes::admin::affiliate::create_rule)
        .put_async("/v1/admin/affiliate/rules/:id", routes::admin::affiliate::update_rule)
        .delete_async("/v1/admin/affiliate/rules/:id", routes::admin::affiliate::delete_rule)
        .get_async("/v1/admin/affiliate/stats", routes::admin::affiliate::stats)
        // Impact
        .get_async("/v1/impact/stats", routes::impact::public_stats)
        .post_async("/v1/impact/record", routes::impact::record)
        .get_async("/v1/admin/impact", routes::impact::admin_stats)
        .run(req, env)
        .await).await
}
