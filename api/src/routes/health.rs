use worker::{Env, Request, Response, Result, RouteContext};
use serde_json::json;

pub async fn handle(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env_name = ctx.env.var("ENVIRONMENT")?.to_string();
    Response::from_json(&json!({
        "status": "ok",
        "environment": env_name,
    }))
}
